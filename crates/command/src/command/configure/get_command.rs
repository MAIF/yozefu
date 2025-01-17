//! Command to fetch a property of the configuration file.
use std::{collections::HashMap, fs};

use crate::command::Command as CliCommand;
use app::Config;
use clap::Args;
use lib::Error;
use serde_json::Value;

#[derive(Debug, Args, Clone)]
pub struct ConfigureGetCommand {
    /// Property you want to read. It must be a JavaScript Object Notation Pointer (RFC 6901) <https://datatracker.ietf.org/doc/html/rfc6901>
    /// Special keywords are also supported: 'config', 'filters', 'logs' etc...
    property: String,
}

impl CliCommand for ConfigureGetCommand {
    async fn execute(&self) -> Result<(), Error> {
        let file = Config::path()?;
        let content = fs::read_to_string(&file)?;
        let config = serde_json::from_str::<Value>(&content)?;
        let mut property_name = self.property.clone();
        if !self.property.starts_with('/') {
            property_name = format!("/{}", property_name);
        }
        match config.pointer(&property_name) {
            Some(p) => {
                println!("{}", serde_json::to_string_pretty(&p)?);
                Ok(())
            }
            None => {
                let config = Config::read(&file)?;
                match self.property.as_str() {
                    "filters" | "filter" | "fn" | "func" | "functions" => {
                        let paths = fs::read_dir(config.filters_dir())?;
                        let mut filters = HashMap::new();
                        for path in paths {
                            let n = path.unwrap();
                            if n.file_type().unwrap().is_file()
                                && n.path().extension().map(|s| s == "wasm").unwrap_or(false)
                            {
                                filters.insert(
                                    n.path().file_stem().unwrap().to_str().unwrap().to_string(),
                                    n.path(),
                                );
                            }
                        }
                        println!("{:}", serde_json::to_string_pretty(&filters)?);
                    }
                    "filter_dir" | "filters_dir" | "filters-dir" | "functions-dir"
                    | "functions_dir" | "function_dir" => {
                        println!("{:?}", config.filters_dir().display())
                    }
                    "log" | "logs" => println!("{:?}", config.logs_file().display()),
                    "configuration_file" | "configuration-file" | "config" | "conf" => {
                        println!("{:?}", file)
                    }
                    "directory" | "dir" => println!("{:?}", file.parent().unwrap()),
                    "themes" => println!("{}", serde_json::to_string_pretty(&config.themes())?),
                    "theme-file" | "themes-file" | "themes_file" | "theme_file" => {
                        println!("{:?}", config.themes_file())
                    }
                    _ => {
                        return Err(Error::Error(format!(
                            "There is no '{}' property in the config file",
                            self.property
                        )))
                    }
                }
                Ok(())
            }
        }
    }
}
