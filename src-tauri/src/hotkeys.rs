use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

pub struct Hotkeys {
    _manager: GlobalHotKeyManager,
}

impl Hotkeys {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let mods = Modifiers::CONTROL | Modifiers::ALT;
        for (code, _slot) in [
            (Code::Digit1, 0u8),
            (Code::Digit2, 1),
            (Code::Digit3, 2),
            (Code::Digit4, 3),
            (Code::Digit5, 4),
            (Code::Digit6, 5),
            (Code::Digit7, 6),
            (Code::Digit8, 7),
            (Code::Digit9, 8),
        ] {
            manager.register(HotKey::new(Some(mods), code))?;
        }
        manager.register(HotKey::new(Some(mods), Code::KeyM))?;
        Ok(Self { _manager: manager })
    }
}
