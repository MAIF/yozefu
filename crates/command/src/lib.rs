//! Structures and utility functions to build the command line interface of `yozefu`.
//! It relies on the [`clap`](https://crates.io/crates/clap) crate.

mod cli;
mod cluster;
mod command;
mod global_args;
mod headless;
mod log;
mod theme;
mod version;
pub use clap::Parser;
pub use cli::Cli;
pub use cluster::Cluster;
pub use global_args::GlobalArgs;
pub use tui::TuiError;

pub use app::APPLICATION_NAME;

//pub fn read_config() -> Result<GlobalConfig, Error> {
//    GlobalConfig::read(&GlobalConfig::path()?)
//}
