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
    /// Process names that, when foregrounded, auto-apply this profile. Each
    /// entry is matched case-insensitively against the foreground executable
    /// filename (e.g. `lol.exe`, `Spotify.exe`). Empty list = manual only.
    #[serde(default)]
    pub app_matches: Vec<String>,
}

fn default_latency() -> u8 { 100 }

pub use store::ProfileStore;
