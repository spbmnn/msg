use anyhow::{Error, Result};

mod app;
mod core;
mod gui;

use app::Msg;
use tracing::{info, level_filters::LevelFilter};

fn main() -> iced::Result {
    init_tracing();
    info!("initialized");

    iced::application(Msg::title, Msg::update, Msg::view)
        .exit_on_close_request(false)
        .subscription(Msg::subscription)
        .run_with(Msg::new)
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_target(true)
        .with_line_number(true)
        .init();
}
