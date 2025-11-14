use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use itertools::Itertools;
use lib::Error;
use serde_json::Value;

use crate::APPLICATION_NAME;

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
/// The workspace is the directory containing yozefu configuration, logs, themes, filters...
pub struct Workspace {
    /// Config directory of Yozefu, the `path` is a directory
    pub path: PathBuf,
    /// Specific config file of Yozefu
    config_file: PathBuf,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            path: Self::yozefu_directory().unwrap(),
            config_file: Self::yozefu_directory()
                .unwrap()
                .join(Self::CONFIG_FILENAME),
        }
    }
}

impl Workspace {
    pub const CONFIG_FILENAME: &str = "config.json";
    pub const LOGS_FILENAME: &str = "application.log";
    pub const THEMES_FILENAME: &str = "themes.json";
    pub const FILTERS_DIRECTORY: &str = "filters";

    pub fn new(directory: &Path, config_file: &Path) -> Self {
        Self {
            path: directory.to_path_buf(),
            config_file: config_file.to_path_buf(),
        }
    }

    /// The default yozefu directory containing themes, filters, config...
    fn yozefu_directory() -> Result<PathBuf, Error> {
        ProjectDirs::from("io", "maif", APPLICATION_NAME)
            .ok_or(Error::Error(
                "Failed to find the yozefu configuration directory".to_string(),
            ))
            .map(|e| e.config_dir().to_path_buf())
    }

    /// Returns the name of config file
    pub fn config_file(&self) -> PathBuf {
        self.config_file.clone()
    }

    /// Returns the name of the logs file
    pub fn logs_file(&self) -> PathBuf {
        self.path.join(Self::LOGS_FILENAME)
    }

    /// Returns the name of the logs file
    pub fn themes_file(&self) -> PathBuf {
        self.path.join(Self::THEMES_FILENAME)
    }

    /// Returns the list of available theme names.
    pub fn themes(&self) -> Vec<String> {
        let file = self.themes_file();
        let content = fs::read_to_string(file).unwrap_or("{}".to_string());
        let themes: HashMap<String, Value> = serde_json::from_str(&content).unwrap_or_default();
        themes
            .keys()
            .map(std::string::ToString::to_string)
            .collect_vec()
    }

    /// Returns the name of the directory containing wasm filters
    pub fn filters_dir(&self) -> PathBuf {
        let dir = self.path.join(Self::FILTERS_DIRECTORY);
        let _ = fs::create_dir_all(&dir);
        dir
    }
}
