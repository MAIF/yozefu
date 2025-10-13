//! The [`clap`](https://crates.io/crates/clap) commands of the tool
use std::env::var;

use lib::Error;

mod config_command;
pub mod configure;
mod create_filter;
mod import_filter;
mod main_command;
mod utility_commands;

pub(crate) use create_filter::CreateFilterCommand;
pub(crate) use import_filter::ImportFilterCommand;
pub use main_command::MainCommand;
pub use utility_commands::UtilityCommands;

#[cfg(target_family = "windows")]
pub const DEFAULT_EDITOR: &str = "notepad";
#[cfg(not(target_family = "windows"))]
pub const DEFAULT_EDITOR: &str = "vim";

/// Trait for all commands.
/// I don't know if it's relevant...
pub trait Command: Send {
    #[allow(async_fn_in_trait)]
    async fn execute(&self) -> Result<(), Error>;
}

fn default_editor(editor: Option<&str>) -> String {
    editor
        .or(var("EDITOR").ok().as_deref())
        .unwrap_or(DEFAULT_EDITOR)
        .to_string()
}
