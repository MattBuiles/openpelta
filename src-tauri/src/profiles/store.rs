use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use super::Profile;

pub struct ProfileStore {
    path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct StoreFile {
    profiles: Vec<Profile>,
    active: Option<String>,
}

impl ProfileStore {
    pub fn new() -> Self {
        let path = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config/openpelta/profiles.toml");
        Self { path }
    }

    pub fn load(&self) -> Result<Vec<Profile>> {
        if !self.path.exists() {
            return Ok(vec![]);
        }
        let raw = fs::read_to_string(&self.path)?;
        let parsed: StoreFile = toml::from_str(&raw)?;
        Ok(parsed.profiles)
    }

    pub fn save(&self, profiles: &[Profile], active: Option<&str>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let store = StoreFile {
            profiles: profiles.to_vec(),
            active: active.map(String::from),
        };
        fs::write(&self.path, toml::to_string_pretty(&store)?)?;
        Ok(())
    }
}
