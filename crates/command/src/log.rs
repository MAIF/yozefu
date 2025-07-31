//! Logging utilities.

use std::{fs::OpenOptions, path::PathBuf};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{SubscriberBuilder, format::FmtSpan},
};

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
    tracing_subscriber_builder(is_debug)
        //.with_writer(std::io::stderr)
        .with_ansi(true)
        .try_init()
}

/// When the user starts the TUI, it writes logs to a file.
pub(crate) fn init_logging_file(
    is_debug: bool,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    if let Err(e) = tracing_subscriber_builder(is_debug)
        .with_writer(OpenOptions::new().append(true).create(true).open(path)?)
        .try_init()
    {
        eprintln!("Failed to initialize logging: {}", e);
    }
    Ok(())
}

/// When the user starts the TUI, it writes logs to a file.
fn tracing_subscriber_builder(is_debug: bool) -> SubscriberBuilder {
    let level = log_level(is_debug);
    let mut builder = tracing_subscriber::fmt().with_max_level(level);
    if is_debug {
        builder = builder.with_span_events(FmtSpan::CLOSE);
    }
    builder
}
