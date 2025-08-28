mod footer_component;
mod header_component;
mod help_component;
mod issue_component;
mod progress_bar_component;
mod record_details_component;
mod records_component;
mod root_component;
mod schemas_component;
mod scroll_state;
mod search_component;
mod shortcut;
mod state;
mod styles;
mod topic_details_component;
mod topics_and_records_component;
mod topics_component;
mod topics_list;
pub mod ui;
mod vertical_scrollable_block;

#[cfg(test)]
use app::configuration::GlobalConfig;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::prelude::Stylize;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType},
};
pub(crate) use root_component::RootComponent;
pub(crate) use shortcut::Shortcut;
use strum::Display;
use tokio::sync::mpsc::UnboundedSender;
pub use ui::Ui;

use std::sync::{Arc, LazyLock, Mutex};

pub use state::State;

use serde::Deserialize;

use crate::{Action, TuiError, records_buffer::RecordsBuffer, tui::Event};

pub(crate) type ConcurrentRecordsBuffer = LazyLock<Arc<Mutex<RecordsBuffer>>>;
static BUFFER: ConcurrentRecordsBuffer =
    LazyLock::new(|| Arc::new(Mutex::new(RecordsBuffer::new())));

#[derive(Debug, Clone, Display, Hash, PartialEq, Eq, Deserialize, PartialOrd, Ord)]
pub(crate) enum ComponentName {
    Records,
    Topics,
    Header,
    Footer,
    RecordDetails,
    TopicsAndRecords,
    RecordsView,
    TopicDetails,
    Main,
    Search,
    Dialog,
    Help,
    Schemas,
}

impl ComponentName {
    pub fn label(&self) -> String {
        match &self {
            ComponentName::RecordDetails => "Record".to_string(),
            ComponentName::TopicDetails => "Topic".to_string(),
            _ => self.to_string(),
        }
    }
}

impl Default for ComponentName {
    fn default() -> Self {
        Self::Topics
    }
}

pub(crate) trait WithHeight: Component {
    fn content_height(&self) -> usize {
        0
    }
}

pub(crate) trait Component: Send + Sync {
    fn register_action_handler(&mut self, _tx: UnboundedSender<Action>) {}

    fn id(&self) -> ComponentName;

    fn make_block_focused_with_state<'a>(&self, state: &State, block: Block<'a>) -> Block<'a> {
        match state.focused == self.id() {
            true => self.make_block_focused(state, block),
            false => block,
        }
    }

    fn make_block_focused<'a>(&self, state: &State, block: Block<'a>) -> Block<'a> {
        block
            .border_style(Style::default().fg(state.theme.focused_border))
            .border_type(BorderType::Thick)
            .title_style(Style::default().bold())
    }

    fn init(&mut self) -> Result<(), TuiError> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>, TuiError> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<Action>, TuiError> {
        Ok(None)
    }

    fn handle_mouse_events(&mut self, _mouse: MouseEvent) -> Result<Option<Action>, TuiError> {
        Ok(None)
    }

    fn update(&mut self, _action: Action) -> Result<Option<Action>, TuiError> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError>;

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![]
    }
}

#[cfg(test)]
pub mod mod_test;
#[cfg(test)]
pub mod records_component_test;
#[cfg(test)]
pub mod root_component_test;

#[cfg(test)]
pub fn default_global_config() -> GlobalConfig {
    use app::configuration::GlobalConfig;

    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_path_buf();

    GlobalConfig {
        path: temp_path.clone().join("config.json"),
        yozefu_directory: temp_path.join("config"),
        logs: None,
        default_url_template: "".to_string(),
        initial_query: "".to_string(),
        theme: "light".to_string(),
        highlighter_theme: None,
        clusters: indexmap::IndexMap::default(),
        default_kafka_config: indexmap::IndexMap::default(),
        history: vec![],
        show_shortcuts: true,
        export_directory: std::path::PathBuf::from(""),
        consumer: Default::default(),
    }
}

#[cfg(test)]
pub fn default_state() -> State {
    use crate::Theme;
    State::new("localhost", Theme::light(), &default_global_config())
}
