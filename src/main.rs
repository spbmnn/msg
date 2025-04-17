mod app;
mod core;
mod gui;
mod util;

use app::App;
use tracing::{error, info, level_filters::LevelFilter};

fn main() -> iced::Result {
    init_tracing();
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

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_line_number(true)
        .init();
}
