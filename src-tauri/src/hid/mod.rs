pub mod protocol;
pub mod pelta;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RgbMode {
    Off,
    Static,
    Breathing,
    Wave,
    Rainbow,
}

pub trait PeltaDevice: Send + Sync {
    fn battery(&self) -> Result<u8>;
    fn set_sidetone(&self, level: u8) -> Result<()>;
    fn set_mic_mute(&self, muted: bool) -> Result<()>;
    fn set_sleep_timer(&self, minutes: u16) -> Result<()>;
    fn firmware_version(&self) -> Result<String>;
    fn set_rgb(&self, mode: RgbMode, color: Rgb, speed: u8) -> Result<()>;
    fn set_eq(&self, bands: &[f32; 10]) -> Result<()>;
}
