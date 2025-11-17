//! This component is handles the main layout of the TUI
//! and renders components based on the current context.
use copypasta::{ClipboardContext, ClipboardProvider};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::warn;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Margin, Rect},
    widgets::{Clear, Paragraph},
};
use tokio::sync::{mpsc::UnboundedSender, watch::Receiver};

use crate::{
    Action, Notification, action::Level, error::TuiError, highlighter::Highlighter,
    records_buffer::BufferAction,
};

use super::{
    Component, ComponentName, ConcurrentRecordsBuffer, State, footer_component::FooterComponent,
    header_component::HeaderComponent, help_component::HelpComponent,
    progress_bar_component::ProgressBarComponent, record_details_component::RecordDetailsComponent,
    records_component::RecordsComponent, schemas_component::SchemasComponent,
    search_component::SearchComponent, topic_details_component::TopicDetailsComponent,
    topics_and_records_component::TopicsAndRecordsComponent, topics_component::TopicsComponent,
};

pub(crate) struct RootComponent {
    components: HashMap<ComponentName, Arc<Mutex<dyn Component>>>,
    views: Vec<ComponentName>,
    state: State,
    focus_history: Vec<ComponentName>,
    focus_order: Vec<ComponentName>,
    progress_bar: ProgressBarComponent,
    buffer_rx: Receiver<BufferAction>,
    action_tx: Option<UnboundedSender<Action>>,
}

impl RootComponent {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(
        query: &str,
        selected_topics: Vec<String>,
        records: &'static ConcurrentRecordsBuffer,
        state: State,
    ) -> Self {
        let config = state.workspace().config();
        let buffer_rx = records.lock().map(|e| e.channels.clone().1).ok().unwrap();
        let mut footer = FooterComponent::default();
        footer.show_shortcuts(config.show_shortcuts);

        let topics_component = Arc::new(Mutex::new(TopicsComponent::new(selected_topics)));

        let highlighter = Highlighter::new(state.highlighter_theme.clone());

        let mut components: [Arc<Mutex<dyn Component>>; 10] = [
            topics_component.clone(),
            Arc::new(Mutex::new(RecordsComponent::new(records))),
            Arc::new(Mutex::new(TopicDetailsComponent::default())),
            Arc::new(Mutex::new(RecordDetailsComponent::new(highlighter.clone()))),
            Arc::new(Mutex::new(SearchComponent::new(
                query,
                config.history.clone(),
                state.workspace().filters_dir(),
            ))),
            Arc::new(Mutex::new(footer)),
            Arc::new(Mutex::new(HelpComponent::default())),
            Arc::new(Mutex::new(SchemasComponent::new(highlighter))),
            Arc::new(Mutex::new(HeaderComponent::default())),
            Arc::new(Mutex::new(FooterComponent::default())),
        ];

        components[components.len() - 1] = Arc::new(Mutex::new(TopicsAndRecordsComponent::new(
            topics_component.clone(),
            components[1].clone(),
        )));

        let components: HashMap<ComponentName, Arc<Mutex<dyn Component>>> = components
            .into_iter()
            .map(|c| {
                let id = c.lock().unwrap().id();
                (id, c)
            })
            .collect();
        Self {
            components,
            buffer_rx,
            progress_bar: ProgressBarComponent::new(400),
            views: vec![ComponentName::TopicsAndRecords],
            focus_order: focus_order_of(&ComponentName::TopicsAndRecords),
            focus_history: vec![],
            state,
            action_tx: None,
        }
    }

    fn focus_next(&mut self, current: &ComponentName) -> ComponentName {
        let index = self
            .focus_order
            .iter()
            .position(|e| e == current)
            .unwrap_or(self.focus_order.len() - 1);
        match index == self.focus_order.len() - 1 {
            true => self.focus_order.first().unwrap().clone(),
            false => self.focus_order[index + 1].clone(),
        }
    }

    fn focus(&mut self, current: ComponentName) -> Result<(), TuiError> {
        self.state.focused = current;
        let mut shortcuts = self
            .components
            .get(&self.state.focused)
            .unwrap()
            .lock()
            .unwrap()
            .shortcuts();

        if self.state.focused == ComponentName::RecordDetails {
            shortcuts.extend(
                self.components
                    .get(&ComponentName::Records)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .shortcuts(),
            );
        }

        self.action_tx
            .as_ref()
            .unwrap()
            .send(Action::Shortcuts(shortcuts, self.views.len() == 1))?;
        Ok(())
    }

    fn close(&mut self) {
        self.views.pop();
        if self.views.is_empty() {
            self.action_tx.as_ref().unwrap().send(Action::Quit).unwrap();
        } else {
            self.focus_order = focus_order_of(self.views.last().unwrap());

            let last_focused_component = self
                .focus_history
                .pop()
                .unwrap_or(self.focus_order.first().unwrap().clone());

            let focus = match self.focus_order.contains(&self.state.focused) {
                true => self.state.focused.clone(),
                false => last_focused_component,
            };
            self.focus(focus).unwrap();
        }
        self.notify_footer().unwrap();
    }

    fn toggle_view(&mut self, view: ComponentName) -> Result<(), TuiError> {
        if self.views.last().unwrap() == &view {
            self.close();
        } else {
            self.action_tx
                .as_ref()
                .unwrap()
                .send(Action::NewView(view))?;
        }
        Ok(())
    }

    fn focus_previous(&mut self, current: &ComponentName) -> ComponentName {
        let index = self
            .focus_order
            .iter()
            .position(|e| e == current)
            .unwrap_or(1);
        match index == 0 {
            true => self.focus_order.last().unwrap().clone(),
            false => self.focus_order[index - 1].clone(),
        }
    }

    fn notify_footer(&self) -> Result<(), TuiError> {
        if !self.views.is_empty() {
            self.action_tx.as_ref().unwrap().send(Action::ViewStack((
                self.views.first().unwrap().clone(),
                self.focus_history.clone(),
            )))?;
        }
        Ok(())
    }
}

impl Component for RootComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx.clone());
        for component in self.components.values_mut() {
            component
                .lock()
                .unwrap()
                .register_action_handler(tx.clone());
        }
    }

    fn id(&self) -> ComponentName {
        ComponentName::Main
    }

    fn init(&mut self) -> Result<(), TuiError> {
        self.notify_footer()?;
        self.action_tx.as_ref().unwrap().send(Action::Shortcuts(
            self.components
                .get(&self.state.focused)
                .unwrap()
                .lock()
                .unwrap()
                .shortcuts(),
            true,
        ))?;
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Tab => {
                let new_focus = self.focus_next(&self.state.focused.clone());
                self.focus(new_focus)?;
            }
            KeyCode::BackTab => {
                let new_focus = self.focus_previous(&self.state.focused.clone());
                self.focus(new_focus)?;
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.action_tx.as_ref().unwrap().send(Action::Refresh)?;
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.focused = ComponentName::Search;
                return Ok(None);
            }
            KeyCode::Char('/' | ':') if self.state.focused != ComponentName::Search => {
                self.state.focused = ComponentName::Search;
                return Ok(None);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.action_tx.as_ref().unwrap().send(Action::Quit)?;
            }
            KeyCode::Char('o')
                if key.modifiers.contains(KeyModifiers::CONTROL) && self.views.len() == 1 =>
            {
                match self.views.first().unwrap() {
                    ComponentName::Records => self.views[0] = ComponentName::TopicsAndRecords,
                    ComponentName::TopicsAndRecords => self.views[0] = ComponentName::Records,
                    _ => warn!(
                        "View '{}' does not support toggling. This instruction should not be unreachable",
                        self.views.first().unwrap()
                    ),
                }
                self.notify_footer()?;
                if self.views.len() == 1 {
                    self.focus_order = focus_order_of(&self.views[0]);
                    self.state.focused = match self.focus_order.contains(&self.state.focused) {
                        true => self.state.focused.clone(),
                        false => self.focus_order.first().unwrap().clone(),
                    };
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::RefreshShortcuts)?;
                }
                return Ok(None);
            }
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.toggle_view(ComponentName::Help)?;
                return Ok(None);
            }
            KeyCode::Esc => self.close(),
            _ => (),
        }
        let focused_component = self.components.get(&self.state.focused).unwrap();
        focused_component.lock().unwrap().handle_key_events(key)?;
        if self.state.focused == ComponentName::RecordDetails
            && (key.code == KeyCode::Up || key.code == KeyCode::Down)
        {
            self.components
                .get(&ComponentName::Records)
                .unwrap()
                .lock()
                .unwrap()
                .handle_key_events(key)?;
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action {
            Action::RefreshShortcuts => {
                let focused = self.state.focused.clone();
                self.focus(focused)?;
            }
            Action::NewView(ref action) => {
                if action == &ComponentName::RecordDetails
                    && self.state.focused == ComponentName::RecordDetails
                {
                    return Ok(None);
                }
                self.focus_history.push(self.state.focused.clone());
                self.focus_history.dedup();
                let last_focused = self.focus_history.last().unwrap().clone();
                self.focus_order = focus_order_of(action);
                self.views = self
                    .views
                    .iter()
                    .filter(|a| a != &action)
                    .cloned()
                    .collect();

                self.views.push(action.clone());
                match self.focus_order.contains(&last_focused) {
                    true => self.focus(last_focused),
                    false => self.focus(
                        self.focus_order
                            .first()
                            .unwrap_or_else(|| {
                                panic!("I think you forgot to define focus order of '{action}'. Please update the 'focus_order_of' function in root_component.rs")
                            })
                            .clone(),
                    ),
                }?;
                self.notify_footer()?;
            }
            Action::RecordsToRead(length) => {
                self.progress_bar.set_length(length);
                let mut a = self.buffer_rx.clone();
                let BufferAction::Stats(mut stats) = *a.borrow_and_update();
                stats.total_to_read = length;
            }
            Action::CopyToClipboard(ref content) => {
                let mut ctx = ClipboardContext::new().unwrap();
                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::Notification(Notification::new(
                        Level::Info,
                        "Copied to clipboard".to_string(),
                    )))?;
                ctx.set_contents(content.to_string()).unwrap();
            }
            _ => (),
        }
        for component in self.components.values_mut() {
            component.lock().unwrap().update(action.clone())?;
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, _: &State) -> Result<(), TuiError> {
        if rect.width < 20 && rect.height < 4 {
            let [area] = Layout::horizontal([Constraint::Length(4)])
                .flex(Flex::Center)
                .areas(rect);
            let [area] = Layout::vertical([Constraint::Length(1)])
                .flex(Flex::Center)
                .areas(area);

            f.render_widget(
                Paragraph::new(
                    String::from_utf8(vec![74, 32, 226, 157, 164, 239, 184, 143]).unwrap(),
                ),
                area,
            );
            return Ok(());
        }
        let mut a = self.buffer_rx.clone();
        let BufferAction::Stats(stats) = *a.borrow_and_update();
        self.progress_bar.set_progress(stats.read);
        f.render_widget(Clear, rect);

        let last_view = self.views.last().unwrap();
        let main_component = self
            .components
            .get(last_view)
            .unwrap_or_else(|| panic!("Unable to find component '{last_view}'"));

        let chunks: std::rc::Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),
                Constraint::Percentage(100),
                Constraint::Min(3),
                Constraint::Min(1),
            ])
            .split(rect.inner(Margin::new(1, 1)));

        main_component
            .lock()
            .unwrap()
            .draw(f, chunks[1], &self.state)?;
        self.components
            .get_mut(&ComponentName::Header)
            .unwrap()
            .lock()
            .unwrap()
            .draw(f, chunks[0], &self.state)?;
        self.components
            .get_mut(&ComponentName::Search)
            .unwrap()
            .lock()
            .unwrap()
            .draw(f, chunks[2], &self.state)?;
        self.components
            .get_mut(&ComponentName::Footer)
            .unwrap()
            .lock()
            .unwrap()
            .draw(f, chunks[3].inner(Margin::new(1, 0)), &self.state)?;

        //f.render_widget(self.progress_bar.clone(), rect);
        //f.render_widget(search_block, chunks[1]);

        Ok(())
    }
}

pub fn focus_order_of(component: &ComponentName) -> Vec<ComponentName> {
    match component {
        ComponentName::RecordDetails => vec![ComponentName::RecordDetails, ComponentName::Search],
        ComponentName::Records => vec![ComponentName::Records, ComponentName::Search],
        ComponentName::Schemas => vec![ComponentName::Schemas, ComponentName::Search],
        ComponentName::TopicsAndRecords => vec![
            ComponentName::Topics,
            ComponentName::Records,
            ComponentName::Search,
        ],
        ComponentName::TopicDetails => vec![ComponentName::TopicDetails, ComponentName::Search],
        ComponentName::Help => vec![ComponentName::Help, ComponentName::Search],
        ComponentName::Dialog
        | ComponentName::Search
        | ComponentName::Main
        | ComponentName::RecordsView
        | ComponentName::Footer
        | ComponentName::Header
        | ComponentName::Topics => vec![],
    }
}
