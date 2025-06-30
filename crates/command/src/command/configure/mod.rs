//! Module gathering the commands related to configuration:
//! - `configure`
//! - `configure get`
//! - `configure set`

use std::{fs, process::Command};

use app::configuration::GlobalConfig;
use clap::{Args, Subcommand};
use lib::Error;
use log::info;
use tempfile::tempdir;

mod get_command;
mod set_command;

pub use get_command::ConfigureGetCommand;
pub use set_command::ConfigureSetCommand;

use super::default_editor;

/// Command to edit the configuration file.
#[derive(Debug, Args, Clone)]
pub struct ConfigureCommand {
    /// Your favorite code editor
    #[clap(short, long)]
    pub editor: Option<String>,
    #[command(subcommand)]
    pub subcommand: Option<ConfigureSubCommand>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum ConfigureSubCommand {
    /// Read a specific property from the config file
    Get(ConfigureGetCommand),
    /// Edit a specific property from the config file
    Set(ConfigureSetCommand),
}

impl crate::command::Command for ConfigureSubCommand {
    async fn execute(&self) -> Result<(), Error> {
        match self {
            ConfigureSubCommand::Get(c) => c.execute().await,
            ConfigureSubCommand::Set(c) => c.execute().await,
        }
    }
}

impl crate::command::Command for ConfigureCommand {
    async fn execute(&self) -> Result<(), Error> {
        if let Some(ref subcommand) = self.subcommand {
            return subcommand.execute().await;
        }

        let file = GlobalConfig::path()?;
        let temp_file =
            tempdir()?.path().join(file.file_name().expect(
                "the configuration path should be a file, not a directory or something else",
            ));
        fs::create_dir_all(temp_file.parent().expect("temp_file.parent() should return something unless the configuration file is at the root of your file system?"))?;
        fs::copy(&file, &temp_file)?;

        let editor = default_editor(&self.editor);
        Command::new(editor)
            .arg(&temp_file)
            .status()
            .expect("Something went wrong");

        let new_config = fs::read_to_string(&temp_file)?;
        fs::remove_file(temp_file)?;
        match serde_json::from_str::<GlobalConfig>(&new_config) {
            Ok(o) => {
                fs::write(&file, serde_json::to_string_pretty(&o)?)?;
                info!(
                    "Configuration file '{}' has been updated successfully.",
                    file.display()
                );
            }
            Err(e) => {
                return Err(Error::Error(format!(
                    "Your new config file is not valid. Please try again: {e:?}"
                )));
            }
        };
        Ok(())
    }
}
