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

/// Download and launch the installer for the host audio EQ backend.
///
/// Equalizer APO is hosted only on SourceForge, which now serves a JS
/// interstitial instead of the binary on its `/download` URLs. The browser
/// follows a `<meta refresh>` tag containing a one-time signed mirror URL —
/// we replicate that here by fetching the interstitial page first, parsing
/// the meta-refresh URL out of the HTML, and then downloading the real
/// binary from it. The installer itself is a GUI wizard so the user still
/// picks the audio device and reboots, but the download + launch step is
/// fully automated.
#[tauri::command]
fn install_eq_backend() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let page_url = "https://sourceforge.net/projects/equalizerapo/files/latest/download";

        // Step 1: pull the interstitial HTML.
        let html_out = std::process::Command::new("curl")
            .args(["-sL", "-A", "Mozilla/5.0 (OpenPelta)", page_url])
            .output()
            .map_err(|e| format!("curl failed to start: {e}"))?;
        if !html_out.status.success() {
            return Err("Could not reach SourceForge to fetch the installer page.".into());
        }
        let html = String::from_utf8_lossy(&html_out.stdout);

        // Step 2: dig the signed mirror URL out of `<meta http-equiv="refresh" ...>`.
        let mirror_url = {
            let after = html
                .split(r#"meta http-equiv="refresh""#)
                .nth(1)
                .ok_or("SourceForge page format changed (no meta refresh)")?;
            let url_start = after.find("url=").ok_or("No url= in meta refresh")? + 4;
            let rest = &after[url_start..];
            let end = rest
                .find('"')
                .ok_or("Malformed meta-refresh URL")?;
            rest[..end].replace("&amp;", "&")
        };

        // Step 3: download the actual installer.
        let installer = std::env::temp_dir().join("EqualizerAPO-installer.exe");
        let installer_str = installer.to_string_lossy().to_string();
        let dl = std::process::Command::new("curl")
            .args([
                "-sLfo", &installer_str,
                "-A", "Mozilla/5.0 (OpenPelta)",
                &mirror_url,
            ])
            .status()
            .map_err(|e| format!("curl download failed: {e}"))?;
        if !dl.success() {
            return Err("Installer download failed — try again in a moment.".into());
        }

        // Sanity check: must be a real PE/EXE (starts with `MZ`).
        let head = std::fs::read(&installer).unwrap_or_default();
        if head.len() < 2 || &head[..2] != b"MZ" {
            return Err(
                "Downloaded file is not an executable; SourceForge may have changed its page format.".into(),
            );
        }

        // Step 4: launch the wizard. APO's installer needs elevation; spawning
        // it directly fails with "requires elevation" (Win32 error 740). Going
        // through PowerShell's `Start-Process -Verb RunAs` pops the standard
        // UAC dialog for the user instead.
        std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-WindowStyle", "Hidden",
                "-Command",
                &format!("Start-Process -FilePath '{}' -Verb RunAs", installer_str),
            ])
            .spawn()
            .map_err(|e| format!("Could not launch installer: {e}"))?;

        Ok("Installer launched — accept the UAC prompt, then walk the wizard (pick the Pelta audio device) and reboot.".into())
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
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;
            use tauri::Manager;

            // When launched by the OS autostart hook, start hidden in the tray
            // instead of popping a window at every boot.
            if std::env::args().any(|a| a == "--autostart") {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            let show = MenuItem::with_id(app, "show", "Show OpenPelta", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("OpenPelta")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        // Hide to tray instead of quitting when the window is closed.
        .on_window_event(|w, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = w.hide();
                api.prevent_close();
            }
        })
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
