pub mod store;

use serde::{Deserialize, Serialize};

use crate::audio::AudioConfig;
use crate::rgb::RgbState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub rgb: RgbState,
    pub eq: AudioConfig,
    #[serde(default)]
    pub nr: bool,
    #[serde(default = "default_latency")]
    pub latency_ms: u8,
}

fn default_latency() -> u8 { 100 }

pub use store::ProfileStore;
