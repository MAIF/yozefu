//! Command that prints the config to `stdout`.
//!
//! ```bash
//! yozf config | jq '.clusters | keys'
//! ```

use app::configuration::GlobalConfig;
use clap::Args;
use lib::Error;
use tracing::info;

use crate::{GlobalArgs, command::Command};

use super::configure::ConfigureSubCommand;

#[derive(Debug, Clone, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub subcommand: Option<ConfigureSubCommand>,
    #[command(flatten)]
    pub global: GlobalArgs,
}

impl Command for ConfigCommand {
    async fn execute(&self) -> Result<(), Error> {
        if let Some(ref subcommand) = self.subcommand {
            return subcommand.execute().await;
        }

        let path = self.global.workspace().config_file();
        info!("The configuration file is located at '{}'", path.display());

        let config = GlobalConfig::read(&path)?;
        println!("{}", serde_json::to_string_pretty(&config)?);

        Ok(())
    }
}
