pub mod audio;
pub mod firmware;
pub mod hid;
pub mod hotkeys;
pub mod profiles;
pub mod rgb;
pub mod tray;
#[cfg(target_os = "windows")]
pub mod win_audio;
#[cfg(target_os = "windows")]
pub mod foreground;

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

/// Fetch a SourceForge installer past their JS interstitial, verify it, and
/// run it elevated. Returns the success message for the frontend.
///
/// SourceForge no longer serves binaries directly from `/projects/<slug>/
/// files/latest/download` — they return a JS page that follows a
/// `<meta http-equiv="refresh">` tag carrying a one-time signed mirror URL.
/// We replicate that flow here so the user just clicks "install" and the
/// elevated wizard pops without manual download steps.
#[cfg(target_os = "windows")]
fn sf_download_and_run(
    project_slug: &str,
    installer_basename: &str,
    success_message: &str,
) -> Result<String, String> {
    let page_url =
        format!("https://sourceforge.net/projects/{project_slug}/files/latest/download");

    let html_out = std::process::Command::new("curl")
        .args(["-sL", "-A", "Mozilla/5.0 (OpenPelta)", &page_url])
        .output()
        .map_err(|e| format!("curl failed to start: {e}"))?;
    if !html_out.status.success() {
        return Err("Could not reach SourceForge to fetch the installer page.".into());
    }
    let html = String::from_utf8_lossy(&html_out.stdout);

    let mirror_url = {
        let after = html
            .split(r#"meta http-equiv="refresh""#)
            .nth(1)
            .ok_or("SourceForge page format changed (no meta refresh)")?;
        let url_start = after.find("url=").ok_or("No url= in meta refresh")? + 4;
        let rest = &after[url_start..];
        let end = rest.find('"').ok_or("Malformed meta-refresh URL")?;
        rest[..end].replace("&amp;", "&")
    };

    let installer = std::env::temp_dir().join(installer_basename);
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

    let head = std::fs::read(&installer).unwrap_or_default();
    if head.len() < 2 || &head[..2] != b"MZ" {
        return Err(
            "Downloaded file is not an executable; SourceForge may have changed its page format."
                .into(),
        );
    }

    // Elevated launch — many installers refuse to run otherwise. PowerShell's
    // `Start-Process -Verb RunAs` pops the UAC prompt the user expects.
    std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-WindowStyle", "Hidden",
            "-Command",
            &format!("Start-Process -FilePath '{}' -Verb RunAs", installer_str),
        ])
        .spawn()
        .map_err(|e| format!("Could not launch installer: {e}"))?;

    Ok(success_message.into())
}

/// Download + launch the Equalizer APO installer.
#[tauri::command]
fn install_eq_backend() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        sf_download_and_run(
            "equalizerapo",
            "EqualizerAPO-installer.exe",
            "Installer launched — accept the UAC prompt, then walk the wizard (pick the Pelta audio device) and reboot.",
        )
    }
    #[cfg(not(target_os = "windows"))]
    { Err("Auto-install is currently Windows-only".into()) }
}

/// Surround 7.1 backend = HeSuVi, which ships HRIR files for APO and a GUI
/// to pick which one to use. Detection looks for the canonical config file
/// it drops next to APO's own `config.txt`.
#[tauri::command]
fn surround_backend_status() -> Option<String> {
    let p = std::path::Path::new(r"C:\Program Files\EqualizerAPO\config\hesuvi-7.1.txt");
    if p.exists() { Some("HeSuVi".into()) } else { None }
}

/// Download + launch the HeSuVi installer. HeSuVi requires APO already
/// installed (it writes files into APO's config dir); we don't enforce that
/// here because the installer itself complains clearly if APO is missing.
#[tauri::command]
fn install_surround_backend() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        sf_download_and_run(
            "hesuvi",
            "HeSuVi-installer.exe",
            "HeSuVi installer launched. Accept the UAC prompt, then open HeSuVi from the Start menu to pick which virtual surround preset you want active.",
        )
    }
    #[cfg(not(target_os = "windows"))]
    { Err("Auto-install is currently Windows-only".into()) }
}

/// Mic mute is a USB Audio Class control, not a vendor command — handled
/// here through Windows Core Audio (default communications capture endpoint).
#[tauri::command]
fn mic_mute() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    { win_audio::get_mic_mute().map_err(|e| e.to_string()) }
    #[cfg(not(target_os = "windows"))]
    { Err("mic mute is currently Windows-only".into()) }
}

#[tauri::command]
fn set_mic_mute(mute: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    { win_audio::set_mic_mute(mute).map_err(|e| e.to_string()) }
    #[cfg(not(target_os = "windows"))]
    { Err("mic mute is currently Windows-only".into()) }
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

/// Push every device-side setting from a profile to the headset (and to the
/// audio backend for EQ). The frontend uses this when the user picks a
/// profile from the dropdown.
#[tauri::command]
fn apply_profile(state: tauri::State<AppState>, profile: Profile) -> Result<(), String> {
    with_device(state.inner(), |d| {
        d.set_rgb(profile.rgb.mode, profile.rgb.color, profile.rgb.speed)?;
        d.set_noise_reduction(profile.nr)?;
        // setLatencyMode rejects values it doesn't know about; skip silently
        // if the saved profile carries something unusual rather than erroring.
        let _ = d.set_latency_mode(profile.latency_ms);
        Ok(())
    })?;
    if let Some(backend) = audio::detect() {
        backend
            .apply(&profile.eq)
            .map_err(|e| format!("EQ backend: {e}"))?;
    }
    Ok(())
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
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
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
            let mic = MenuItem::with_id(app, "tray_mic", "Toggle mic mute", true, None::<&str>)?;
            let nr = MenuItem::with_id(app, "tray_nr", "Toggle noise reduction", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &sep, &mic, &nr, &sep, &quit])?;

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
                    "tray_mic" => {
                        #[cfg(target_os = "windows")]
                        if let Ok(cur) = win_audio::get_mic_mute() {
                            let _ = win_audio::set_mic_mute(!cur);
                        }
                    }
                    "tray_nr" => {
                        let state: tauri::State<AppState> = app.state();
                        let _ = with_device(state.inner(), |d| {
                            let cur = d.noise_reduction()?;
                            d.set_noise_reduction(!cur)
                        });
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

            // Global hotkeys: Ctrl+Alt+M (mic mute) and Ctrl+Alt+1..9
            // (switch to saved profile slot N). The manager isn't Send+Sync
            // (platform handles, interior mutability), so instead of stashing
            // it in app state we leak it — its only job is staying alive for
            // the lifetime of the program so the registrations don't drop.
            match hotkeys::install(&app.handle()) {
                Ok(h) => { Box::leak(Box::new(h)); }
                Err(e) => tracing::warn!("Could not register global hotkeys: {e}"),
            }

            // Foreground-window watcher for per-app auto-profile switching.
            #[cfg(target_os = "windows")]
            foreground::install(&app.handle());

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
            mic_mute,
            set_mic_mute,
            audio_backend_status,
            install_eq_backend,
            surround_backend_status,
            install_surround_backend,
            set_eq,
            list_profiles,
            save_profiles,
            apply_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
