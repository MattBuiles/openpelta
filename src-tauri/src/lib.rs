pub mod audio;
pub mod firmware;
pub mod hid;
pub mod hotkeys;
pub mod profiles;
pub mod rgb;
pub mod tray;

use std::sync::Mutex;

use audio::AudioConfig;
use hid::{Pelta, PeltaDevice, PowerInfo, Rgb, RgbMode};
use profiles::{Profile, ProfileStore};

pub struct AppState {
    pub device: Mutex<Option<Pelta>>,
    pub store: ProfileStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            device: Mutex::new(Pelta::open().ok()),
            store: ProfileStore::new(),
        }
    }
}

/// Run a closure against the device, with lazy (re)open and auto-recovery:
/// if the handle is None (never opened or dropped after a previous error),
/// try to open it; if the closure fails, drop the handle so the next call
/// retries from scratch — this covers the headset being unplugged/replugged
/// or the wireless link dropping out while the app keeps running.
fn with_device<T>(
    state: &AppState,
    f: impl FnOnce(&Pelta) -> anyhow::Result<T>,
) -> Result<T, String> {
    let mut guard = state.device.lock().unwrap();
    if guard.is_none() {
        *guard = Pelta::open().ok();
    }
    let dev = guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?;
    match f(dev) {
        Ok(v) => Ok(v),
        Err(e) => {
            // Drop the stale handle so the next operation retries from scratch.
            *guard = None;
            Err(e.to_string())
        }
    }
}

// ---------- reads ----------

#[tauri::command]
fn firmware_version(state: tauri::State<AppState>) -> Result<String, String> {
    with_device(state.inner(), |d| d.firmware_version())
}

#[tauri::command]
fn power_info(state: tauri::State<AppState>) -> Result<PowerInfo, String> {
    with_device(state.inner(), |d| d.power_info())
}

#[tauri::command]
fn headset_present(state: tauri::State<AppState>) -> Result<bool, String> {
    with_device(state.inner(), |d| d.headset_present())
}

#[tauri::command]
fn led_enabled(state: tauri::State<AppState>) -> Result<bool, String> {
    with_device(state.inner(), |d| d.led_enabled())
}

#[tauri::command]
fn noise_reduction(state: tauri::State<AppState>) -> Result<bool, String> {
    with_device(state.inner(), |d| d.noise_reduction())
}

#[tauri::command]
fn latency_mode(state: tauri::State<AppState>) -> Result<u8, String> {
    with_device(state.inner(), |d| d.latency_mode())
}

#[tauri::command]
fn sidetone(state: tauri::State<AppState>) -> Result<(u8, bool), String> {
    with_device(state.inner(), |d| Ok((d.sidetone_volume()?, d.sidetone_enabled()?)))
}

// ---------- writes ----------

#[tauri::command]
fn set_led_color(state: tauri::State<AppState>, color: Rgb) -> Result<(), String> {
    with_device(state.inner(), |d| d.set_led_color(color))
}

#[tauri::command]
fn set_rgb(
    state: tauri::State<AppState>,
    mode: RgbMode,
    color: Rgb,
    intensity: u8,
) -> Result<(), String> {
    with_device(state.inner(), |d| d.set_rgb(mode, color, intensity))
}

#[tauri::command]
fn set_light_effect(
    state: tauri::State<AppState>,
    mode: u8,
    intensity: u8,
    color: Rgb,
) -> Result<(), String> {
    with_device(state.inner(), |d| d.set_light_effect(mode, intensity, color))
}

#[tauri::command]
fn set_noise_reduction(state: tauri::State<AppState>, on: bool) -> Result<(), String> {
    with_device(state.inner(), |d| d.set_noise_reduction(on))
}

#[tauri::command]
fn set_latency_mode(state: tauri::State<AppState>, value_ms: u8) -> Result<(), String> {
    with_device(state.inner(), |d| d.set_latency_mode(value_ms))
}

#[tauri::command]
fn set_demo_mode(state: tauri::State<AppState>, on: bool) -> Result<(), String> {
    with_device(state.inner(), |d| d.set_demo_mode(on))
}

/// Force a fresh open of the device — used by the UI after a reconnect.
#[tauri::command]
fn reconnect(state: tauri::State<AppState>) -> Result<bool, String> {
    let mut guard = state.device.lock().unwrap();
    *guard = Pelta::open().ok();
    Ok(guard.is_some())
}

/// Returns the name of the installed audio EQ backend, or `None` if none is
/// found (detection is cheap — runs on every call so newly-installed backends
/// are picked up without restarting the app).
#[tauri::command]
fn audio_backend_status() -> Option<String> {
    audio::detect().map(|b| b.name().to_string())
}

/// Launch the installer for the platform's audio EQ backend. On Windows this
/// runs `winget install` for Equalizer APO — the user has to walk through the
/// installer (it asks which audio device to bind to) and reboot afterwards.
#[tauri::command]
fn install_eq_backend() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("winget")
            .args([
                "install", "-e", "--id", "peters.EqualizerAPO",
                "--accept-source-agreements", "--accept-package-agreements",
            ])
            .spawn()
            .map_err(|e| format!("Could not launch winget: {e}"))?;
        Ok("Installer launched. Pick your Pelta audio device when prompted, then reboot.".into())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Auto-install is currently Windows-only".into())
    }
}

/// EQ does not flow through the device — it is applied by the host audio
/// backend (Equalizer APO on Windows, EasyEffects on Linux). Returns an error
/// if no supported backend is installed.
#[tauri::command]
fn set_eq(config: AudioConfig) -> Result<(), String> {
    let backend = audio::detect()
        .ok_or_else(|| "No audio EQ backend installed (Equalizer APO / EasyEffects)".to_string())?;
    backend.apply(&config).map_err(|e| e.to_string())
}

// ---------- profiles ----------

#[tauri::command]
fn list_profiles(state: tauri::State<AppState>) -> Result<Vec<Profile>, String> {
    state.store.load().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_profiles(
    state: tauri::State<AppState>,
    profiles: Vec<Profile>,
    active: Option<String>,
) -> Result<(), String> {
    state
        .store
        .save(&profiles, active.as_deref())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            reconnect,
            firmware_version,
            power_info,
            headset_present,
            led_enabled,
            noise_reduction,
            latency_mode,
            sidetone,
            set_led_color,
            set_rgb,
            set_light_effect,
            set_noise_reduction,
            set_latency_mode,
            set_demo_mode,
            audio_backend_status,
            install_eq_backend,
            set_eq,
            list_profiles,
            save_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
