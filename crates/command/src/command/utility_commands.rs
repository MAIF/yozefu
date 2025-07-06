//! Other CLI commands to make the tool user-friendly.

use clap::Subcommand;
use lib::Error;

use crate::log::init_logging_stderr;

use super::{
    Command, CreateFilterCommand, ImportFilterCommand, config_command::ConfigCommand,
    configure::ConfigureCommand,
};

#[derive(Subcommand, Debug)]
pub enum UtilityCommands {
    /// Import a search filter
    #[clap(aliases = ["register-filter"])]
    ImportFilter(ImportFilterCommand),
    /// Helper to create a new WebAssembly search filter
    #[clap(alias = "new-filter")]
    CreateFilter(CreateFilterCommand),
    /// Edit the configuration file
    Configure(ConfigureCommand),
    /// Print the config to `stdout`
    Config(ConfigCommand),
}

impl Command for UtilityCommands {
    async fn execute(&self) -> Result<(), Error> {
        let _ = init_logging_stderr(false);
        match self {
            Self::ImportFilter(command) => command.execute().await,
            Self::CreateFilter(command) => command.execute().await,
            Self::Configure(command) => command.execute().await,
            Self::Config(command) => command.execute().await,
        }
    }
}
