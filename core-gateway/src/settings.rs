use anyhow::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub listen: String,
    pub openai_upstream: Option<String>,
    pub anthropic_upstream: Option<String>,
    pub tls_listen: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:8777".to_string(),
            openai_upstream: None,
            anthropic_upstream: None,
            tls_listen: Some("127.0.0.1:9443".to_string()),
        }
    }
}

impl Settings {
    pub fn config_dir() -> PathBuf {
        if let std::result::Result::Ok(d) = std::env::var("AIPROXY_DATA_DIR") {
            return PathBuf::from(d);
        }
        let proj = ProjectDirs::from("local", "ai-proxy", "gateway").unwrap();
        proj.config_dir().to_path_buf()
    }

    pub fn load(path: Option<&str>) -> Result<Self> {
        if let Some(p) = path {
            let content = fs::read_to_string(p)?;
            let cfg: Settings = toml::from_str(&content)?;
            return Ok(cfg);
        }
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let file = dir.join("config.toml");
        if file.exists() {
            let content = fs::read_to_string(&file)?;
            let cfg: Settings = toml::from_str(&content)?;
            return Ok(cfg);
        }
        let cfg = Settings::default();
        let text = toml::to_string_pretty(&cfg)?;
        fs::write(&file, text)?;
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let file = dir.join("config.toml");
        let text = toml::to_string_pretty(self)?;
        std::fs::write(file, text)?;
        Ok(())
    }
}
