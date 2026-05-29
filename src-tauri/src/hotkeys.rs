//! Global hotkeys. Ctrl+Alt+M toggles mic mute (handled here directly so it
//! reacts even when the window is hidden). Ctrl+Alt+1..9 switch to the
//! N-th saved profile — those are dispatched as Tauri events the frontend
//! listens to, so the UI stays in sync with the device.

use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

pub struct Hotkeys {
    _manager: GlobalHotKeyManager,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    ToggleMicMute,
    Profile(u8),
}

pub fn install(app: &AppHandle) -> Result<Hotkeys> {
    let manager = GlobalHotKeyManager::new()?;
    let mods = Modifiers::CONTROL | Modifiers::ALT;

    let mut by_id: HashMap<u32, Action> = HashMap::new();
    let digit_codes = [
        Code::Digit1, Code::Digit2, Code::Digit3, Code::Digit4, Code::Digit5,
        Code::Digit6, Code::Digit7, Code::Digit8, Code::Digit9,
    ];
    for (i, code) in digit_codes.iter().enumerate() {
        let hk = HotKey::new(Some(mods), *code);
        let id = hk.id();
        manager.register(hk)?;
        by_id.insert(id, Action::Profile(i as u8));
    }
    let mute = HotKey::new(Some(mods), Code::KeyM);
    let mute_id = mute.id();
    manager.register(mute)?;
    by_id.insert(mute_id, Action::ToggleMicMute);

    let app = app.clone();
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        for event in receiver.iter() {
            if event.state != HotKeyState::Pressed {
                continue;
            }
            if let Some(action) = by_id.get(&event.id).copied() {
                dispatch(&app, action);
            }
        }
    });

    Ok(Hotkeys { _manager: manager })
}

fn dispatch(app: &AppHandle, action: Action) {
    match action {
        Action::ToggleMicMute => {
            #[cfg(target_os = "windows")]
            if let Ok(cur) = crate::win_audio::get_mic_mute() {
                let new_state = !cur;
                if crate::win_audio::set_mic_mute(new_state).is_ok() {
                    let _ = app.emit("hotkey:mic-mute", new_state);
                }
            }
        }
        Action::Profile(i) => {
            let _ = app.emit("hotkey:profile", i);
        }
    }
}
