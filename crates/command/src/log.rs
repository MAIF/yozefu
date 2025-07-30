//! Logging utilities.

use std::{fs::OpenOptions, path::PathBuf};
use tracing_subscriber::filter::LevelFilter;

/// Returns the log level based on the debug flag.
fn log_level(is_debug: bool) -> LevelFilter {
    match is_debug {
        true => LevelFilter::DEBUG,
        false => LevelFilter::INFO,
    }
}

/// When the user starts the headless mode, it writes logs to `stderr`.
pub(crate) fn init_logging_stderr(
    is_debug: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let level = log_level(is_debug);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .try_init()?;
    Ok(())
}

/// When the user starts the TUI, it writes logs to a file.
pub(crate) fn init_logging_file(
    is_debug: bool,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let level = log_level(is_debug);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(OpenOptions::new().append(true).create(true).open(path)?)
        .try_init()
}
