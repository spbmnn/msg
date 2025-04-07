use anyhow::Result;

mod app;
mod core;
mod gui;

use app::Msg;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    iced::application(Msg::title, Msg::update, Msg::view).run_with(Msg::new)?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_line_number(true)
        .init();
}
