pub mod store;

use serde::{Deserialize, Serialize};

use crate::audio::AudioConfig;
use crate::rgb::RgbState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub audio: AudioConfig,
    pub rgb: RgbState,
    pub sidetone: u8,
    pub surround: bool,
}

pub use store::ProfileStore;
