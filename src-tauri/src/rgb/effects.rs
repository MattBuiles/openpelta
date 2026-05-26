use serde::{Deserialize, Serialize};

use crate::hid::{Rgb, RgbMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbState {
    pub mode: RgbMode,
    pub color: Rgb,
    pub speed: u8,
}

impl Default for RgbState {
    fn default() -> Self {
        Self {
            mode: RgbMode::Static,
            color: Rgb { r: 255, g: 0, b: 0 },
            speed: 50,
        }
    }
}
