//! The state is a struct containing various information.
//! It is passed to all components.
use app::configuration::{InternalConfig, Workspace};
use std::path::PathBuf;

use crate::{highlighter::Highlighter, theme::Theme};

use super::ComponentName;

#[derive(Clone)]
pub struct State {
    pub(crate) focused: ComponentName,
    pub cluster: String,
    pub themes: Vec<String>,
    pub theme: Theme,
    pub internal_config: InternalConfig,
    pub highlighter_theme: Option<syntect::highlighting::Theme>,
    pub configuration_file: PathBuf,
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
                config.workspace().config().highlighter_theme.as_deref(),
            ),
            internal_config: config.clone(),
            themes: config.workspace().themes(),
            configuration_file: config.workspace().config_file(),
            config: config.clone(),
        }
    }

    pub fn workspace(&self) -> &Workspace {
        self.config.workspace()
    }

    pub(crate) fn is_focused(&self, component_name: &ComponentName) -> bool {
        &self.focused == component_name
    }
}
