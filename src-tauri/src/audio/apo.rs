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
    fn name(&self) -> &'static str {
        "Equalizer APO"
    }

    fn apply(&self, cfg: &AudioConfig) -> Result<()> {
        if !self.config_dir.exists() {
            return Err(anyhow!("Equalizer APO not installed at {:?}", self.config_dir));
        }
        let our_file = self.config_dir.join("openpelta.txt");
        fs::write(&our_file, Self::render(cfg))?;

        // Make sure APO's main config sources our preset, otherwise nothing
        // we write here actually reaches the audio chain. Idempotent.
        let main = self.config_dir.join("config.txt");
        let include_line = "Include: openpelta.txt";
        let current = fs::read_to_string(&main).unwrap_or_default();
        if !current.contains(include_line) {
            let updated = if current.is_empty() {
                format!("{include_line}\n")
            } else {
                format!("{}\n{include_line}\n", current.trim_end())
            };
            fs::write(&main, updated)?;
        }
        Ok(())
    }

    fn is_installed(&self) -> bool {
        self.config_dir.exists()
    }
}
