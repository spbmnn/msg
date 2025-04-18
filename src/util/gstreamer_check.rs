use gstreamer as gst;
use std::error::Error;
use tracing::{debug, error, info};

/// Check if essential GStreamer plugins are loaded properly.
pub fn verify_gstreamer_plugins() -> Result<(), Box<dyn Error>> {
    info!("checking gstreamer plugins");
    gst::init()?;

    let required_elements = ["videoconvert", "autovideosink", "decodebin", "playbin"];

    for name in &required_elements {
        if gst::ElementFactory::find(name).is_none() {
            error!("Missing required GStreamer plugin: {}", name);
            return Err(format!("Missing GStreamer plugin: {}", name).into());
        }
    }

    info!("GStreamer core plugins loaded successfully.");
    Ok(())
}
