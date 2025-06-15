use std::io::Cursor;
use std::path::PathBuf;

use directories::ProjectDirs;
use gstreamer::glib::object::{Cast, ObjectExt};
use gstreamer::prelude::*;
use gstreamer::{self as gst};
use gstreamer_app::AppSink;
use iced::advanced::image::Bytes;
use iced::widget::image::Handle;
use iced_video_player::Video;
use image::DynamicImage;
use thiserror::Error;
use tracing::{debug, instrument, trace, warn};
use url::Url;

use crate::core::model::Sample;

use super::http::CLIENT;
use super::model::File;

const SIZE: u32 = 4096; // Textures larger than 4096x4096 tend to crash wgpu

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("Missing preview URL")]
    MissingUrl,

    //#[error("Couldn't convert path to URL")]
    //UrlConvertError,
    #[error("Encoding error")]
    EncodingFailed,

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Video error: {0}")]
    VideoError(#[from] iced_video_player::Error),

    #[error("Gif error: {0}")]
    GifError(#[from] iced_gif::gif::Error),

    //#[error("Pipeline media error")]
    //PipelineTypeError,
    #[error("Pipeline error: {0}")]
    BoolError(#[from] gst::glib::BoolError),

    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
}

#[instrument(skip(url))]
pub async fn fetch_preview(id: u32, url: String) -> Result<Handle, MediaError> {
    trace!("Fetching preview for {id}");
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

#[instrument(skip(file))]
pub async fn fetch_image(id: u32, file: File) -> Result<Handle, MediaError> {
    let id = id;
    let ext = file.ext.unwrap_or("jpg".to_string());
    let original_path: PathBuf = cache_dir().join("original").join(format!("{id}.{ext}"));
    let resized_path: PathBuf = cache_dir().join("resized").join(format!("{id}.png"));

    if resized_path.exists() {
        trace!("Loading {id} from cache ({resized_path:?})");
        let bytes = std::fs::read(resized_path)?;
        return Ok(Handle::from_bytes(bytes));
    }

    let original_bytes = if original_path.exists() {
        std::fs::read(&original_path)?
    } else {
        let url = file.url.as_ref().ok_or(MediaError::MissingUrl)?;
        trace!("Getting post {id} from server ({url})");
        let bytes = CLIENT.get(url).send().await?.bytes().await?.into();

        trace!("Saving to {original_path:?}");
        std::fs::create_dir_all(original_path.parent().unwrap())?;
        std::fs::write(&original_path, &bytes)?;
        bytes
    };

    let img = image::load_from_memory(&original_bytes)?;

    let resized: DynamicImage = if img.width() > SIZE || img.height() > SIZE {
        debug!("Resizing {}x{} to {SIZE}x{SIZE}", img.width(), img.height());
        img.resize(SIZE, SIZE, image::imageops::FilterType::Triangle) // NOTE: maybe make this adjustable?
    } else {
        img
    };

    let mut buf = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|err| {
            warn!("Failed to resize image for post {}: {err}", id);
            MediaError::EncodingFailed
        })?;
    std::fs::create_dir_all(resized_path.parent().unwrap())?;
    std::fs::write(&resized_path, &buf)?;

    Ok(Handle::from_bytes(buf))
}

#[instrument(skip(file))]
pub async fn fetch_sample(id: u32, file: Sample) -> Result<Handle, MediaError> {
    let path: PathBuf = cache_dir().join("sample").join(format!("{id}.jpg"));

    if path.exists() {
        trace!("Loading {id} from cache ({path:?})");
        let bytes = std::fs::read(path)?;
        return Ok(Handle::from_bytes(bytes));
    }

    let url = file.url.as_ref().ok_or(MediaError::MissingUrl)?;
    trace!("Getting post {id} from server ({url})");
    let bytes: Bytes = CLIENT.get(url).send().await?.bytes().await?;
    trace!("Saving to {path:?}");
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, &bytes)?;

    Ok(Handle::from_bytes(bytes))
}

#[instrument(skip(url))]
pub async fn fetch_gif(id: u32, url: String) -> Result<Vec<u8>, MediaError> {
    let file_path: PathBuf = cache_dir().join("gifs").join(format!("{}.gif", id));

    let bytes: Vec<u8> = if file_path.exists() {
        trace!("Loading {id} from cache ({file_path:?})");
        std::fs::read(file_path)?
    } else {
        trace!("Getting {id} from server ({url})");
        let bytes = CLIENT.get(url).send().await?.bytes().await?.into();

        debug!("Saving to {file_path:?}");
        std::fs::create_dir_all(file_path.parent().unwrap())?;
        std::fs::write(&file_path, &bytes)?;
        bytes
    };
    Ok(bytes.to_vec())
}

#[instrument(skip(url))]
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

/// For building video/audio player.
#[instrument]
pub fn build_video_pipeline(uri: &str) -> Result<Video, anyhow::Error> {
    gst::init()?;

    let pipeline = gst::Pipeline::with_name("video-pipeline");

    let src = gst::ElementFactory::make("uridecodebin")
        .property("uri", &uri)
        .build()?;

    let videoscale = gst::ElementFactory::make("videoscale").build()?;
    let videoconvert = gst::ElementFactory::make("videoconvert").build()?;
    let appsink_element = gst::ElementFactory::make("appsink")
        .name("iced_video")
        .build()?;

    let appsink = appsink_element
        .clone()
        .dynamic_cast::<AppSink>()
        .expect("Failed to cast to AppSink");

    appsink.set_property("emit-signals", &true);
    appsink.set_property("sync", &true);
    appsink.set_property(
        "caps",
        &gst::Caps::builder("video/x-raw")
            .field("format", &"NV12")
            .build(),
    );

    let audioconvert = gst::ElementFactory::make("audioconvert").build()?;
    let audioresample = gst::ElementFactory::make("audioresample").build()?;
    let audiosink = gst::ElementFactory::make("autoaudiosink").build()?;

    pipeline.add_many(&[
        &src,
        &videoscale,
        &videoconvert,
        &appsink_element,
        &audioconvert,
        &audioresample,
        &audiosink,
    ])?;

    // Video pad linking
    let videoconvert_clone = videoconvert.clone();
    src.connect_pad_added(move |_src, pad| {
        let sink_pad = videoconvert_clone.static_pad("sink").unwrap();
        if !sink_pad.is_linked() {
            let _ = pad.link(&sink_pad);
        }
    });

    gst::Element::link_many(&[&videoconvert, &appsink_element])?;

    // Audio pad linking
    let audioconvert_clone = audioconvert.clone();
    src.connect_pad_added(move |_src, pad| {
        if pad.current_caps().map_or(false, |caps| {
            caps.structure(0)
                .map_or(false, |s| s.name().starts_with("audio/"))
        }) {
            let sink_pad = audioconvert_clone.static_pad("sink").unwrap();
            if !sink_pad.is_linked() {
                let _ = pad.link(&sink_pad);
            }
        }
    });

    gst::Element::link_many(&[&audioconvert, &audioresample, &audiosink])?;

    let video = Video::from_gst_pipeline(pipeline, appsink, None)?;
    Ok(video)
}

pub fn cache_dir() -> PathBuf {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .unwrap()
        .cache_dir()
        .to_path_buf()
}

pub fn thumbnail_dir() -> PathBuf {
    cache_dir().join("thumbnails")
}

pub fn sample_dir() -> PathBuf {
    cache_dir().join("sample")
}

pub fn image_dir() -> PathBuf {
    cache_dir().join("resized")
}

pub fn gif_dir() -> PathBuf {
    cache_dir().join("gifs")
}

pub fn video_dir() -> PathBuf {
    cache_dir().join("video")
}
