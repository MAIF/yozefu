//! Structures and utility functions to build the command line interface of `yozefu`.
//! It relies on the [`clap`](https://crates.io/crates/clap) crate.

mod cli;
mod command;
mod headless;
mod log;
use app::configuration::GlobalConfig;
pub use clap::Parser;
pub use cli::Cli;
use lib::Error;
pub use tui::TuiError;

pub use app::APPLICATION_NAME;

pub fn read_config() -> Result<GlobalConfig, Error> {
    GlobalConfig::read(&GlobalConfig::path()?)
}
