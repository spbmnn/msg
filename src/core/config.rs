use core::fmt;
use directories::ProjectDirs;
use iced::Theme;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use thiserror::Error;
use tracing::info;

use super::{blacklist::Blacklist, model::FollowedTag};

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

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq)]
pub enum MsgTheme {
    Light,
    #[default]
    Dark,
}

impl MsgTheme {
    pub fn get(&self) -> Theme {
        match self {
            MsgTheme::Dark => Theme::Dark,
            MsgTheme::Light => Theme::Light,
        }
    }
}

impl ToString for MsgTheme {
    fn to_string(&self) -> String {
        match self {
            MsgTheme::Dark => "Dark",
            MsgTheme::Light => "Light",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub auth: Option<Auth>,
    pub blacklist: Blacklist,
    pub followed_tags: Vec<FollowedTag>,
    #[serde(default)]
    pub theme: MsgTheme,
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

        info!("Loaded config from {path:?}");
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
