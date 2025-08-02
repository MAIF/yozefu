//! Logging utilities.

use std::{fs::OpenOptions, path::PathBuf};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter,
    filter::Directive,
    fmt::{
        SubscriberBuilder,
        format::{DefaultFields, FmtSpan},
    },
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
        .with_writer(std::io::stderr)
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
        eprintln!("Failed to initialize logging: {e}");
    }
    Ok(())
}

/// When the user starts the TUI, it writes logs to a file.
fn tracing_subscriber_builder(
    is_debug: bool,
) -> SubscriberBuilder<DefaultFields, tracing_subscriber::fmt::format::Format, EnvFilter> {
    let level = log_level(is_debug);
    let mut filter = EnvFilter::try_from_default_env().unwrap_or(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::ERROR.into())
            .from_env_lossy(),
    );
    if is_debug {
        filter = filter.add_directive(Directive::from(level))
    }

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(true)
}
