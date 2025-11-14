//! Command to fetch a property of the configuration file.
use std::{collections::HashMap, fs};

use crate::{GlobalArgs, command::Command as CliCommand, theme::update_themes};
use app::configuration::GlobalConfig;
use clap::Args;
use lib::Error;
use serde_json::Value;
use tui::HIGHLIGHTER_THEMES;

#[derive(Debug, Args, Clone)]
pub struct ConfigureGetCommand {
    /// Property you want to read. It must be a JavaScript Object Notation Pointer (RFC 6901) <https://datatracker.ietf.org/doc/html/rfc6901>
    /// Special keywords are also supported: 'config', 'filters', 'logs' etc...
    property: String,
    #[clap(flatten)]
    pub global: GlobalArgs,
}

impl CliCommand for ConfigureGetCommand {
    async fn execute(&self) -> Result<(), Error> {
        let workspace = self.global.workspace();
        let file = workspace.config_file();
        let content = fs::read_to_string(&file)?;
        let config = serde_json::from_str::<Value>(&content)?;
        let mut property_name = self.property.clone();
        if !self.property.starts_with('/') {
            property_name = format!("/{property_name}");
        }
        match config.pointer(&property_name) {
            Some(p) => {
                println!("{}", serde_json::to_string_pretty(&p)?);
                Ok(())
            }
            None => {
                let config = GlobalConfig::read(&file)?;
                match self.property.as_str() {
                    "filters" | "filter" | "fn" | "func" | "functions" => {
                        let paths = fs::read_dir(workspace.filters_dir())?;
                        let mut filters = HashMap::new();
                        for path in paths {
                            let n = path.unwrap();
                            if n.file_type().unwrap().is_file()
                                && n.path().extension().is_some_and(|s| s == "wasm")
                            {
                                filters.insert(
                                    n.path().file_stem().unwrap().to_str().unwrap().to_string(),
                                    n.path(),
                                );
                            }
                        }
                        println!("{}", serde_json::to_string_pretty(&filters)?);
                    }
                    "path" | "file" => println!("{}", config.path.display()),
                    "filter_dir" | "filters_dir" | "filters-dir" | "functions-dir"
                    | "functions_dir" | "function_dir" => {
                        println!("{}", workspace.filters_dir().display());
                    }
                    "log" | "logs" => println!("{}", config.logs_file().display()),
                    "configuration_file" | "configuration-file" | "config" | "conf" => {
                        println!("{}", file.display());
                    }
                    "directory" | "dir" => println!("{}", file.parent().unwrap().display()),
                    "themes" => {
                        let mut output = HashMap::new();
                        let _ = update_themes(&workspace).await;
                        output.insert("themes", serde_json::to_value(workspace.themes())?);
                        output.insert("highlighter", serde_json::to_value(HIGHLIGHTER_THEMES)?);
                        println!("{}", serde_json::to_string_pretty(&output)?);
                    }
                    "theme-file" | "themes-file" | "themes_file" | "theme_file" => {
                        println!("{}", workspace.themes_file().display());
                    }
                    _ => {
                        return Err(Error::Error(format!(
                            "There is no '{}' property in the config file",
                            self.property
                        )));
                    }
                }
                Ok(())
            }
        }
    }
}
