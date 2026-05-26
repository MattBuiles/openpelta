pub mod audio;
pub mod firmware;
pub mod hid;
pub mod hotkeys;
pub mod profiles;
pub mod rgb;
pub mod tray;

use std::sync::Mutex;

use hid::{Pelta, PeltaDevice, Rgb, RgbMode};
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

#[tauri::command]
fn battery(state: tauri::State<AppState>) -> Result<u8, String> {
    let guard = state.device.lock().unwrap();
    guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?
        .battery()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_sidetone(state: tauri::State<AppState>, level: u8) -> Result<(), String> {
    let guard = state.device.lock().unwrap();
    guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?
        .set_sidetone(level)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_mic_mute(state: tauri::State<AppState>, muted: bool) -> Result<(), String> {
    let guard = state.device.lock().unwrap();
    guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?
        .set_mic_mute(muted)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_rgb(
    state: tauri::State<AppState>,
    mode: RgbMode,
    color: Rgb,
    speed: u8,
) -> Result<(), String> {
    let guard = state.device.lock().unwrap();
    guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?
        .set_rgb(mode, color, speed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_eq(state: tauri::State<AppState>, bands: [f32; 10]) -> Result<(), String> {
    let guard = state.device.lock().unwrap();
    if let Some(dev) = guard.as_ref() {
        dev.set_eq(&bands).map_err(|e| e.to_string())?;
    }
    // Also push to system audio backend (APO/EasyEffects) — TODO once AudioConfig wiring complete
    Ok(())
}

#[tauri::command]
fn firmware_version(state: tauri::State<AppState>) -> Result<String, String> {
    let guard = state.device.lock().unwrap();
    guard
        .as_ref()
        .ok_or_else(|| "Pelta not connected".to_string())?
        .firmware_version()
        .map_err(|e| e.to_string())
}

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
            battery,
            set_sidetone,
            set_mic_mute,
            set_rgb,
            set_eq,
            firmware_version,
            list_profiles,
            save_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
