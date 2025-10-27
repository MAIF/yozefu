//! The state is a struct containing various information.
//! It is passed to all components.
use app::configuration::InternalConfig;
use std::path::PathBuf;

use crate::{highlighter::Highlighter, theme::Theme};

use super::ComponentName;

#[derive(Clone)]
pub struct State {
    pub(crate) focused: ComponentName,
    pub cluster: String,
    pub themes: Vec<String>,
    pub theme: Theme,
    pub highlighter_theme: Option<syntect::highlighting::Theme>,
    pub configuration_file: PathBuf,
    pub logs_file: PathBuf,
    pub themes_file: PathBuf,
    pub filters_dir: PathBuf,
    pub config: InternalConfig,
}

impl State {
    pub fn new(cluster: &str, theme: Theme, config: &InternalConfig) -> Self {
        let temp = theme.highlighter_theme.clone();
        Self {
            focused: ComponentName::default(),
            cluster: cluster.to_string(),
            theme,
            highlighter_theme: Highlighter::theme(
                temp.as_deref(),
                config.global.highlighter_theme.as_deref(),
            ),
            themes: config.global.themes(),
            themes_file: config.global.themes_file(),
            configuration_file: config.global.path.clone(),
            logs_file: config.global.logs_file(),
            filters_dir: config.global.filters_dir(),
            config: config.clone(),
        }
    }

    pub(crate) fn is_focused(&self, component_name: &ComponentName) -> bool {
        &self.focused == component_name
    }
}
