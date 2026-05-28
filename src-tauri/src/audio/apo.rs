use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// `fs::write`, but retried a few times if APO has the file open (it holds
/// `config.txt` briefly when its file watcher fires). Without this the first
/// `Apply EQ` after launch frequently fails with "process cannot access the
/// file because it is being used by another process" (Win32 error 32).
fn write_retry(path: &Path, content: &str) -> Result<()> {
    let mut delay_ms = 50u64;
    for attempt in 0..6 {
        match fs::write(path, content) {
            Ok(()) => return Ok(()),
            Err(e) if attempt < 5 && e.raw_os_error() == Some(32) => {
                thread::sleep(Duration::from_millis(delay_ms));
                delay_ms = (delay_ms * 2).min(500);
            }
            Err(e) => return Err(e.into()),
        }
    }
    unreachable!()
}

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
        write_retry(&our_file, &Self::render(cfg))?;

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
            write_retry(&main, &updated)?;
        }
        Ok(())
    }

    fn is_installed(&self) -> bool {
        self.config_dir.exists()
    }
}
