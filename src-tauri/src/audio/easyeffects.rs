use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use super::{AudioBackend, AudioConfig};

pub struct EasyEffects {
    preset_dir: PathBuf,
}

impl EasyEffects {
    pub fn new() -> Self {
        let dir = dirs_path()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config/easyeffects/output");
        Self { preset_dir: dir }
    }
}

fn dirs_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

impl AudioBackend for EasyEffects {
    fn apply(&self, cfg: &AudioConfig) -> Result<()> {
        fs::create_dir_all(&self.preset_dir)?;
        let path = self.preset_dir.join("openpelta.json");

        let bands: Vec<_> = cfg
            .bands
            .iter()
            .enumerate()
            .map(|(i, b)| {
                (
                    format!("band{}", i),
                    json!({
                        "frequency": b.freq_hz,
                        "gain": b.gain_db,
                        "q": b.q,
                        "type": "Bell",
                        "mode": "RLC (BT)",
                    }),
                )
            })
            .collect();

        let mut eq_obj = serde_json::Map::new();
        eq_obj.insert("input-gain".into(), json!(cfg.preamp_db));
        eq_obj.insert("num-bands".into(), json!(cfg.bands.len()));
        for (k, v) in bands {
            eq_obj.insert(k, v);
        }

        let preset = json!({
            "output": {
                "plugins_order": if cfg.surround_enabled {
                    vec!["equalizer", "convolver"]
                } else {
                    vec!["equalizer"]
                },
                "equalizer": eq_obj,
            }
        });

        fs::write(&path, serde_json::to_string_pretty(&preset)?)?;
        Ok(())
    }

    fn is_installed(&self) -> bool {
        which("easyeffects").is_some()
    }
}

fn which(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let p = dir.join(bin);
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        })
    })
}
