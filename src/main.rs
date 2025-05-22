mod app;
mod core;
mod gui;
mod util;

use app::App;
use tracing::{error, info};

fn main() -> iced::Result {
    crate::core::tracing::init_tracing();
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    info!("Starting {name} v{version}");

    if let Err(e) = util::gstreamer_check::verify_gstreamer_plugins() {
        error!("GStreamer check failed: {e}");
        std::process::exit(1);
    }

    iced::application(app::title, app::update, app::view)
        .theme(app::theme)
        .exit_on_close_request(false)
        .subscription(app::subscription)
        .run_with(App::new)
}
