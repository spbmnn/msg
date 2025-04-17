use gstreamer as gst;
use std::error::Error;
use tracing::{error, info};

/// Check if essential GStreamer plugins are loaded properly.
pub fn verify_gstreamer_plugins() -> Result<(), Box<dyn Error>> {
    info!("checking gstreamer plugins");
    gst::init()?;

    let registry = gst::Registry::get();
    let required_plugins = ["playback", "videoconvert", "autovideosink", "decodebin"];

    for name in &required_plugins {
        if registry.find_plugin(name).is_none() {
            error!("Missing required GStreamer plugin: {}", name);
            return Err(format!("Missing GStreamer plugin: {}", name).into());
        }
    }

    info!("GStreamer core plugins loaded successfully.");
    Ok(())
}
