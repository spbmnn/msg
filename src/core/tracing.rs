use std::path::PathBuf;

use directories::ProjectDirs;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("msg=debug"));

    let stdout_layer = fmt::layer().with_target(true);

    let file = open_log_file().unwrap();
    let file_layer = fmt::layer()
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();
}

pub fn log_path() -> Option<PathBuf> {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .map(|dirs| dirs.data_local_dir().join("msg.log"))
}

pub fn open_log_file() -> std::io::Result<std::fs::File> {
    let path = log_path().expect("Couldn't get log path");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::File::create(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_resolved() {
        let path = log_path().expect("should resolve");
        assert!(path.ends_with("msg.log"));
    }
}
