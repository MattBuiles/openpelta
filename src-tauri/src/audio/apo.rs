use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

use super::{AudioBackend, AudioConfig};

const DEFAULT_PATH: &str = r"C:\Program Files\EqualizerAPO\config";

pub struct EqualizerApo {
    config_dir: PathBuf,
}

impl EqualizerApo {
    pub fn new() -> Self {
        Self {
            config_dir: PathBuf::from(DEFAULT_PATH),
        }
    }

    fn render(cfg: &AudioConfig) -> String {
        let mut out = String::new();
        out.push_str("Device: ROG Pelta\n");
        out.push_str(&format!("Preamp: {:.1} dB\n", cfg.preamp_db));
        for (i, b) in cfg.bands.iter().enumerate() {
            out.push_str(&format!(
                "Filter {}: ON PK Fc {} Hz Gain {:.1} dB Q {:.2}\n",
                i + 1,
                b.freq_hz,
                b.gain_db,
                b.q
            ));
        }
        if cfg.surround_enabled {
            out.push_str("Include: hesuvi-7.1.txt\n");
        }
        out
    }
}

impl AudioBackend for EqualizerApo {
    fn apply(&self, cfg: &AudioConfig) -> Result<()> {
        if !self.config_dir.exists() {
            return Err(anyhow!("Equalizer APO not installed at {:?}", self.config_dir));
        }
        let path = self.config_dir.join("openpelta.txt");
        fs::write(&path, Self::render(cfg))?;
        Ok(())
    }

    fn is_installed(&self) -> bool {
        self.config_dir.exists()
    }
}
