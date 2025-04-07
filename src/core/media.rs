use std::io::Write;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use iced::widget::image::Handle;
use iced_gif::Frames;
use iced_video_player::Video;
use thiserror::Error;
use url::Url;

use super::http::CLIENT;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Missing preview URL")]
    MissingUrl,

    #[error("IO Error")]
    IOError(#[from] std::io::Error),

    #[error("{0}")]
    VideoError(#[from] iced_video_player::Error),

    #[error("{0}")]
    GifError(#[from] iced_gif::gif::Error),

    #[error("Request failed")]
    Request(#[from] reqwest::Error),
}

pub async fn fetch_preview(url: String) -> Result<Handle, MediaError> {
    let bytes = CLIENT.get(url).send().await?.bytes().await?;

    Ok(Handle::from_bytes(bytes.to_vec()))
}

pub async fn fetch_gif(id: u32, url: String) -> Result<Vec<u8>, MediaError> {
    let bytes = CLIENT.get(url).send().await?.bytes().await?;

    Ok(bytes.to_vec())
}

pub async fn fetch_video(id: u32, url: String, ext: String) -> Result<Url, MediaError> {
    let file_path: PathBuf = data_path().unwrap().join(format!("{}.{}", id, ext));
    let mut file = std::fs::File::create(&file_path)?;

    let bytes = CLIENT.get(url).send().await?.bytes().await?;
    file.write_all(&bytes)?;

    let url: Result<Url, ()> = Url::from_file_path(&file_path);
    match url {
        Ok(url) => Ok(url),
        Err(()) => Err(MediaError::MissingUrl),
    }
}

pub async fn fetch_image(id: u32, url: String, ext: String) -> Result<Handle, MediaError> {
    let bytes = CLIENT.get(url).send().await?.bytes().await?;

    Ok(Handle::from_bytes(bytes.to_vec()))
}

fn data_path() -> Option<PathBuf> {
    match ProjectDirs::from("xyz", "stripywalrus", "msg") {
        Some(path) => Some(path.data_dir().to_path_buf()),
        None => None,
    }
}
