mod app;
mod core;
mod gui;
mod util;

use app::App;
use clap::{command, Arg};
use tracing::{error, info};

fn main() -> iced::Result {
    crate::core::tracing::init_tracing();
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    info!("Starting {name} v{version}");

    let matches = command!()
        .arg(
            Arg::new("debug")
                .short('d')
                .help("Enable debug view")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let debug = matches.get_flag("debug");
    if debug {
        info!("Debug view enabled");
    }

    if let Err(e) = util::gstreamer_check::verify_gstreamer_plugins() {
        error!("GStreamer check failed: {e}");
        std::process::exit(1);
    }

    iced::application(app::title, app::update, app::view)
        .theme(app::theme)
        .exit_on_close_request(false)
        .subscription(app::subscription)
        .run_with(move || App::new(debug))
}
