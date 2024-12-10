//! Command to edit the configuration file.

use std::fs;

use crate::{cli::config_path, command::Command as CliCommand};
use app::Config;
use clap::Args;
use lib::Error;
use log::info;
use serde_json::Value;

#[derive(Debug, Args, Clone)]
pub struct ConfigureSetCommand {
    /// Property you want to edit. It must be a JavaScript Object Notation Pointer (RFC 6901) <https://datatracker.ietf.org/doc/html/rfc6901>
    property: String,
    /// Its new value
    value: String,
}

impl CliCommand for ConfigureSetCommand {
    async fn execute(&self) -> Result<(), Error> {
        let file = config_path();

        let content = fs::read_to_string(&file)?;
        let mut config = serde_json::from_str::<Value>(&content)?;
        let mut property_name = self.property.clone();
        if !self.property.starts_with('/') {
            property_name = format!("/{}", property_name);
        }
        let property = config.pointer_mut(&property_name);
        if property.is_none() {
            return Err(Error::Error(format!(
                "Property '{}' does not exist",
                self.property
            )));
        }
        let old_value = property.unwrap();
        let new_value =
            serde_json::from_str(&self.value).unwrap_or(Value::String(self.value.clone()));
        match std::mem::discriminant(old_value) == std::mem::discriminant(&new_value) {
            true => {
                let _  = std::mem::replace(old_value, new_value);
                info!("'{}' is now equal to '{}'", property_name, old_value);
                let c: Config = serde_json::from_value(config)?;
                fs::write(file, serde_json::to_string_pretty(&c)?)?;
                Ok(())
            },
            false => Err(Error::Error(format!("Old value is '{}'. The new value is '{}'. As you can see, these are 2 different types", old_value, new_value))),
        }
    }
}
