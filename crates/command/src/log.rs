//! Logging utilities.

use std::{fs::OpenOptions, path::PathBuf};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

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
    tracing_subscriber::fmt()
        .with_max_level(log_level(is_debug))
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .try_init()
}

/// When the user starts the TUI, it writes logs to a file.
pub(crate) fn init_logging_file(
    _is_debug: bool,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let filter = EnvFilter::try_from_default_env().unwrap_or(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::ERROR.into())
            .from_env_lossy(),
    );

    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(OpenOptions::new().append(true).create(true).open(path)?)
        .try_init()
    {
        eprintln!("Failed to initialize logging: {e}");
    }
    Ok(())
}
