use core::fmt;
use directories::ProjectDirs;
use iced::Theme;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use thiserror::Error;
use tracing::info;

use super::blacklist::Blacklist;

const fn _default_true() -> bool {
    true
}
const fn _default_false() -> bool {
    false
}
const fn default_ppr() -> usize {
    5
}
const fn default_tile_width() -> usize {
    180
}

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
#[serde(default)]
pub struct Config {
    pub auth: Option<Auth>,
    pub blacklist: Blacklist,
    pub followed_tags: FxHashMap<String, Option<u32>>,
    pub view: ViewConfig,
}

#[derive(Deserialize, Default, Serialize, Clone)]
pub struct Auth {
    pub username: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewConfig {
    #[serde(default)]
    pub theme: MsgTheme,
    #[serde(default = "default_ppr")]
    pub posts_per_row: usize,
    #[serde(default = "default_tile_width")]
    pub tile_width: usize,
    #[serde(default = "_default_false")]
    pub download_sample: bool,
    #[serde(default = "_default_true")]
    pub download_fullsize: bool,
}

impl Default for ViewConfig {
    fn default() -> Self {
        ViewConfig {
            theme: MsgTheme::default(),
            posts_per_row: 5,
            tile_width: 180,
            download_sample: false,
            download_fullsize: true,
        }
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("username", &self.username)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

pub fn config_path() -> Result<PathBuf, ConfigError> {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or(ConfigError::LoadPathError)
}

impl Config {
    pub fn new() -> Config {
        Config::load().unwrap_or_default()
    }

    pub fn load() -> Result<Config, ConfigError> {
        let path = config_path()?.join("config.toml");

        let raw = fs::read_to_string(&path)?;
        let config = toml::from_str::<Config>(&raw)?;

        info!("Loaded config from {path:?}");
        return Ok(config);
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path()?.join("config.toml");
        fs::create_dir_all(path.parent().unwrap())?;
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        Ok(())
    }
}
