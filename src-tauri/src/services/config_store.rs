use crate::models::config::AppConfig;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct ConfigStore {
    path: PathBuf,
    data: AppConfig,
}

impl ConfigStore {
    pub fn new() -> Result<Self> {
        let dir = directories::ProjectDirs::from("com", "bilitool", "BiliLiveTool")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project dirs"))?;
        let config_dir = dir.config_dir();
        fs::create_dir_all(config_dir)?;
        let path = config_dir.join("config.toml");

        let data = if path.exists() {
            let content = fs::read_to_string(&path)?;
            match toml::from_str(&content) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!("Config file corrupted at {}: {}. Using defaults.", path.display(), e);
                    AppConfig::default()
                }
            }
        } else {
            AppConfig::default()
        };

        Ok(Self { path, data })
    }

    pub fn data(&self) -> &AppConfig {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut AppConfig {
        &mut self.data
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.data)?;
        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, &self.path)?;
        Ok(())
    }
}
