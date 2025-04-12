use core::fmt;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use thiserror::Error;

use super::model::FollowedTag;

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("couldn't get config path")]
    LoadPathError,

    #[error("IO error")]
    IOError(#[from] std::io::Error),

    #[error("TOML deserialize error")]
    TomlDeError(#[from] toml::de::Error),

    #[error("TOML serialize error")]
    TomlSerError(#[from] toml::ser::Error),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub auth: Option<Auth>,
    pub blacklist: Blacklist,
    pub followed_tags: Vec<FollowedTag>,
}

#[derive(Deserialize, Default, Serialize, Clone)]
pub struct Auth {
    pub username: String,
    pub api_key: String,
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("username", &self.username)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct Blacklist {
    pub rules: Vec<String>,
}

fn config_path() -> Result<PathBuf, ConfigError> {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .map(|dirs| dirs.config_dir().join(FILE_NAME))
        .ok_or(ConfigError::LoadPathError)
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

    pub fn load() -> Result<Config, ConfigError> {
        let path = config_path()?;

        let raw = fs::read_to_string(&path)?;
        let config = toml::from_str::<Config>(&raw)?;
        return Ok(config);
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        Ok(())
    }
}
