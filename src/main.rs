mod app;
mod core;
mod gui;
mod util;

use app::App;
use tracing::{error, info};

fn main() -> iced::Result {
    crate::core::tracing::init_tracing();
    info!("logging initialized");

    if let Err(e) = util::gstreamer_check::verify_gstreamer_plugins() {
        error!("GStreamer check failed: {e}");
        std::process::exit(1);
    }

    iced::application("MSG", app::update, app::view)
        .exit_on_close_request(false)
        .subscription(app::subscription)
        .run_with(App::new)
}
