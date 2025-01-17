//! Structures and utility functions to build the command line interface of `yozefu`.
//! It relies on the [`clap`](https://crates.io/crates/clap) crate.

mod cli;
mod command;
mod headless;
mod log;
pub use clap::Parser;
pub use cli::Cli;
pub use tui::TuiError;

pub use app::APPLICATION_NAME;
