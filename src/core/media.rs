use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use iced::advanced::graphics::geometry::frame::Backend;
use iced::advanced::graphics::image::image_rs::EncodableLayout;
use iced::widget::image::Handle;
use iced_gif::Frames;
use iced_video_player::Video;
use image::DynamicImage;
use thiserror::Error;
use tracing::{debug, info, trace};
use url::Url;

use super::http::CLIENT;

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Missing preview URL")]
    MissingUrl,

    #[error("Couldn't convert path to URL")]
    UrlConvertError,

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Video error: {0}")]
    VideoError(#[from] iced_video_player::Error),

    #[error("Gif error: {0}")]
    GifError(#[from] iced_gif::gif::Error),

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

}

pub async fn fetch_preview(id: u32, url: String) -> Result<Handle, MediaError> {
    let file_path: PathBuf = cache_dir().join("thumbnails").join(format!("{}.jpg", id));

    if file_path.exists() {
        trace!("Loading {id} from cache ({file_path:?})");
        let bytes = std::fs::read(file_path)?;
        return Ok(Handle::from_bytes(bytes));
    }

    trace!("Getting {id} from server ({url})");
    let bytes = CLIENT.get(url).send().await?.bytes().await?;

    trace!("Saving to {file_path:?}");
    std::fs::create_dir_all(file_path.parent().unwrap())?;
    std::fs::write(&file_path, &bytes)?;

    Ok(Handle::from_bytes(bytes.to_vec()))
}

pub async fn fetch_image(id: u32, url: String, ext: String) -> Result<Handle, MediaError> {
    let file_path: PathBuf = cache_dir().join("images").join(format!("{}.{}", id, ext));

    let bytes = if file_path.exists() {
        std::fs::read(file_path)?
    } else {
    trace!("Getting {id} from server ({url})");
        let web_bytes = CLIENT.get(url).send().await?.bytes().await?.into();

        trace!("Saving to {file_path:?}");
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        std::fs::write(&file_path, &web_bytes)?;
        web_bytes
    };


    let img = image::load_from_memory(&bytes)?;

    let clamped: DynamicImage = if img.width() > 4096 || img.height() > 4096 {
        img.resize(4096, 4096, image::imageops::FilterType::Triangle) // NOTE: maybe make this adjustable?
    } else {
        img
    };

    let mut buf = Vec::new();
    clamped.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

    Ok(Handle::from_bytes(buf))
}

pub async fn fetch_gif(id: u32, url: String) -> Result<Vec<u8>, MediaError> {
    let file_path: PathBuf = cache_dir().join("gifs").join(format!("{}.gif", id));

    let mut bytes: Vec<u8> = vec![];

    if file_path.exists() {
        trace!("Loading {id} from cache ({file_path:?})");
        bytes = std::fs::read(file_path)?;
    } else {
        trace!("Getting {id} from server ({url})");
        bytes = CLIENT.get(url).send().await?.bytes().await?.into();

        trace!("Saving to {file_path:?}");
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        std::fs::write(&file_path, &bytes)?;
    }



    Ok(bytes.to_vec())
}

pub async fn fetch_video(id: u32, url: String, ext: String) -> Result<Url, MediaError> {
    let file_path: PathBuf = cache_dir().join("video").join(format!("{}.{}", id, ext));

    if file_path.exists() {
        trace!("Loading {id} from cache ({file_path:?})");
    } else {
        trace!("Getting {id} from server ({url})");
        let bytes = CLIENT.get(url).send().await?.bytes().await?;

        trace!("Saving to {file_path:?}");
        std::fs::create_dir_all(&file_path.parent().unwrap())?;
        std::fs::write(&file_path, &bytes)?;
    }

    let url: Result<Url, ()> = Url::from_file_path(&file_path);
    match url {
        Ok(url) => Ok(url),
        Err(()) => Err(MediaError::MissingUrl),
    }
}


fn cache_dir() -> PathBuf {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .unwrap()
        .cache_dir()
        .to_path_buf()
}
