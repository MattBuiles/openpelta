pub mod audio;
pub mod firmware;
pub mod hid;
pub mod hotkeys;
pub mod profiles;
pub mod rgb;
pub mod tray;

use std::sync::Mutex;

use audio::{AudioBackend, AudioConfig};
use hid::{Pelta, PeltaDevice, PowerInfo, Rgb, RgbMode};
use profiles::{Profile, ProfileStore};

pub struct AppState {
    pub device: Mutex<Option<Pelta>>,
    pub audio: Option<Box<dyn AudioBackend>>,
    pub store: ProfileStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            device: Mutex::new(Pelta::open().ok()),
            audio: audio::detect(),
            store: ProfileStore::new(),
        }
    }
}

/// Run a closure against the connected device, mapping the absence and any
/// device error into a `String` for the frontend.
fn with_device<T>(
    state: &AppState,
    f: impl FnOnce(&Pelta) -> anyhow::Result<T>,
) -> Result<T, String> {
    let guard = state.device.lock().unwrap();
    let dev = guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?;
    f(dev).map_err(|e| e.to_string())
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

/// EQ does not flow through the device — it is applied by the host audio
/// backend (Equalizer APO on Windows, EasyEffects on Linux). Returns an error
/// if no supported backend is installed.
#[tauri::command]
fn set_eq(state: tauri::State<AppState>, config: AudioConfig) -> Result<(), String> {
    let backend = state
        .audio
        .as_ref()
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
            set_eq,
            list_profiles,
            save_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
