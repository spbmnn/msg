use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use thiserror::Error;

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("couldn't get config path")]
    LoadPathError,

    #[error("IO error")]
    IOError(#[from] std::io::Error),

    #[error("TOML error")]
    TomlError(#[from] toml::de::Error),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub username: Option<String>,
    pub api_key: Option<String>,
}

fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("xyz", "stripywalrus", "msg").map(|dirs| dirs.config_dir().join(FILE_NAME))
}

impl Config {
    pub fn new() -> Config {
        let path = config_path().unwrap();

        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<Config>(&raw) {
                return config;
            }
        }

        Config::default()
    }

    pub fn load(&self) -> Result<Config, ConfigError> {
        let path = config_path().ok_or(ConfigError::LoadPathError)?;

        let raw = fs::read_to_string(&path)?;
        let config = toml::from_str::<Config>(&raw)?;
        return Ok(config);
    }

    pub fn save(config: &Config) {
        if let Some(path) = config_path() {
            let _ = fs::create_dir_all(path.parent().unwrap());
            if let Ok(toml) = toml::to_string_pretty(config) {
                let _ = fs::write(path, toml);
            }
        }
    }
}
