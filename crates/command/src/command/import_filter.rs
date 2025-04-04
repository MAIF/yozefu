//! Command to import a search filter.
use std::{fs, path::PathBuf};

use app::{
    configuration::GlobalConfig,
    search::filter::{MATCHES_FUNCTION_NAME, PARSE_PARAMETERS_FUNCTION_NAME},
};
use clap::Args;
use extism::{Manifest, Plugin, Wasm};
use lib::Error;
use log::info;

use crate::command::Command;

/// Import a search filter.
/// It also checks that it complies with the tool requirements.
#[derive(Debug, Clone, Args)]
pub(crate) struct ImportFilterCommand {
    /// Search filter to import
    file: PathBuf,
    /// Name of the search filter
    #[clap(short, long = "name")]
    filter_name: Option<String>,
    /// Overwrite the search filter file if it already exists
    #[clap(long)]
    force: bool,
}

/// Wasm functions a search filter must expose.
pub const REQUIRED_WASM_FUNCTIONS: [&str; 2] =
    [PARSE_PARAMETERS_FUNCTION_NAME, MATCHES_FUNCTION_NAME];

impl Command for ImportFilterCommand {
    async fn execute(&self) -> Result<(), Error> {
        let destination = self.destination()?;
        let name = self.name();
        if fs::metadata(&destination).is_ok() && !self.force {
            return Err(Error::Error(format!(
                "The wasm function '{}' already exists. If you want to import it again, please delete it first or use the '--force' flag.",
                destination.display()
            )));
        }

        self.check_wasm_module(&self.file)?;
        fs::copy(&self.file, &destination)?;
        info!("'{}' has been imported successfully", destination.display());
        info!("To use it: `from begin offset > 50 && {}(...)`", name);

        Ok(())
    }
}

impl ImportFilterCommand {
    /// Returns the path to the wasm file.
    pub fn destination(&self) -> Result<PathBuf, Error> {
        let name = self.name();
        let config = GlobalConfig::read(&GlobalConfig::path()?)?;
        let dir = config.filters_dir();
        fs::create_dir_all(&dir)?;
        Ok(dir.join(format!("{}.wasm", name)))
    }

    /// Returns the name of the search filter.
    pub fn name(&self) -> String {
        match &self.filter_name {
            Some(name) => name.to_string(),
            None => self.file.file_stem().unwrap().to_str().unwrap().to_string(),
        }
    }

    /// Checks that the search filter complies with the tool requirements.
    /// The search filter must expose functions defined in `REQUIRED_WASM_FUNCTIONS`.
    fn check_wasm_module(&self, wasm_file: &PathBuf) -> Result<(), Error> {
        let url = Wasm::file(wasm_file);
        let manifest = Manifest::new([url]);
        let mut filter =
            Plugin::new(manifest, [], true).map_err(|e| Error::Error(e.to_string()))?;
        check_presence_of_functions(&mut filter)?;
        Ok(())
    }
}

fn check_presence_of_functions(plugin: &mut Plugin) -> Result<(), Error> {
    for function_name in REQUIRED_WASM_FUNCTIONS.iter() {
        match plugin.function_exists(function_name) {
            true => info!("'{}' found in the search filter", function_name),
            false => {
                return Err(Error::Error(format!(
                    "'{}' is missing in the search filter. Make sure the wasm module exports a '{}' filter",
                    function_name, function_name
                )));
            }
        }
    }
    Ok(())
}

//fn check_parse_parameters(plugin: &mut Plugin) -> Result<(), Error> {
//    match plugin
//        .call::<String, i32>(PARSE_PARAMETERS_FUNCTION_NAME, "[]".to_string())
//        .map(|e| e == 0)
//    {
//        Ok(_) => Ok(()),
//        Err(e) => Err(Error::Error(e.to_string())),
//    }
//}
