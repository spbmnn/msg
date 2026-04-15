mod app;
mod core;
mod gui;
mod util;

use app::App;
use clap::{command, Arg};
use iced::window::{icon, Settings};
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

    iced::application(App::new, App::update, App::view)
        .window(Settings {
            icon: icon::from_file_data(include_bytes!("msg.png"), None).ok(),
            ..Settings::default()
        })
        .theme(App::theme)
        .exit_on_close_request(false)
        .subscription(App::subscription)
        .title(App::title)
        .run()
}
