//! The state is a struct containing various information.
//! It is passed to all components.
use app::Config;
use std::path::PathBuf;

use crate::theme::Theme;

use super::ComponentName;

#[derive(Clone)]
pub struct State {
    pub focused: ComponentName,
    pub cluster: String,
    pub themes: Vec<String>,
    pub theme: Theme,
    pub configuration_file: PathBuf,
    pub logs_file: PathBuf,
    pub themes_file: PathBuf,
    pub filters_dir: PathBuf,
}

impl State {
    pub fn new(cluster: &str, theme: Theme, config: &Config) -> Self {
        Self {
            focused: ComponentName::default(),
            cluster: cluster.to_string(),
            theme,
            themes: config.themes(),
            themes_file: config.themes_file(),
            configuration_file: config.path.clone(),
            logs_file: config.logs_file(),
            filters_dir: config.filters_dir(),
        }
    }

    pub fn is_focused(&self, component_name: ComponentName) -> bool {
        self.focused == component_name
    }
}
