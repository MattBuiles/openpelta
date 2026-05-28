pub mod apo;
pub mod easyeffects;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBand {
    pub freq_hz: u32,
    pub gain_db: f32,
    pub q: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub bands: Vec<EqBand>,
    pub preamp_db: f32,
    pub surround_enabled: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

pub trait AudioBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, cfg: &AudioConfig) -> Result<()>;
    fn is_installed(&self) -> bool;
}

pub fn detect() -> Option<Box<dyn AudioBackend>> {
    #[cfg(target_os = "windows")]
    {
        let b = apo::EqualizerApo::new();
        if b.is_installed() {
            return Some(Box::new(b));
        }
    }
    #[cfg(target_os = "linux")]
    {
        let b = easyeffects::EasyEffects::new();
        if b.is_installed() {
            return Some(Box::new(b));
        }
    }
    None
}
