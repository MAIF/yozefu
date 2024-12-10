//! This component is handles the main layout of the TUI
//! and renders components based on the current context.
use app::Config;
use copypasta::{ClipboardContext, ClipboardProvider};
use lib::ExportedKafkaRecord;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    widgets::Clear,
    Frame,
};
use tokio::sync::{mpsc::UnboundedSender, watch::Receiver};

use crate::{error::TuiError, records_buffer::BufferAction, Action, Notification};

use super::{
    footer_component::FooterComponent, help_component::HelpComponent,
    progress_bar_component::ProgressBarComponent, record_details_component::RecordDetailsComponent,
    records_component::RecordsComponent, search_component::SearchComponent,
    topic_details_component::ScrollableTopicDetailsComponent,
    topics_and_records_component::TopicsAndRecordsComponent, topics_component::TopicsComponent,
    Component, ComponentName, ConcurrentRecordsBuffer, State,
};

pub struct RootComponent {
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
        query: String,
        selected_topics: Vec<String>,
        config: &Config,
        records: &'static ConcurrentRecordsBuffer,
        state: State,
    ) -> Self {
        let buffer_rx = records.lock().map(|e| e.channels.clone().1).ok().unwrap();
        let mut footer = FooterComponent::default();
        footer.show_shortcuts(config.show_shortcuts);

        let mut components: [Arc<Mutex<dyn Component>>; 8] = [
            Arc::new(Mutex::new(TopicsComponent::new(selected_topics))),
            Arc::new(Mutex::new(RecordsComponent::new(records))),
            Arc::new(Mutex::new(ScrollableTopicDetailsComponent::default())),
            Arc::new(Mutex::new(RecordDetailsComponent::new(&state))),
            Arc::new(Mutex::new(SearchComponent::new(
                &query,
                config.history.clone(),
            ))),
            Arc::new(Mutex::new(footer)),
            Arc::new(Mutex::new(HelpComponent::default())),
            Arc::new(Mutex::new(FooterComponent::default())),
        ];

        components[components.len() - 1] = Arc::new(Mutex::new(TopicsAndRecordsComponent::new(
            components[0].clone(),
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
            action_tx: Default::default(),
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
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), TuiError> {
        self.action_tx = Some(tx.clone());
        for component in self.components.values_mut() {
            component
                .lock()
                .unwrap()
                .register_action_handler(tx.clone())?;
        }
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

    fn id(&self) -> ComponentName {
        ComponentName::Main
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
            KeyCode::Char('/') | KeyCode::Char(':')
                if self.state.focused != ComponentName::Search =>
            {
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
                    _ => unreachable!("nope nope"),
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
        };
        let focused_component = self.components.get(&self.state.focused).unwrap();
        focused_component.lock().unwrap().handle_key_events(key)?;
        if self.state.focused == ComponentName::RecordDetails {
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
                    false => self.focus(self.focus_order.first().unwrap().clone()),
                }?;
                self.notify_footer()?;
            }
            Action::RecordsToRead(length) => {
                self.progress_bar.set_length(length);
            }
            Action::Close(_) => self.close(),
            Action::CopyToClipboard(ref record) => {
                let mut ctx = ClipboardContext::new().unwrap();
                let exported_record: ExportedKafkaRecord = record.into();
                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::Notification(Notification::new(
                        log::Level::Info,
                        "Copied to clipboard".to_string(),
                    )))?;
                ctx.set_contents(serde_json::to_string_pretty(&exported_record)?)
                    .unwrap();
            }
            _ => (),
        }
        for component in self.components.values_mut() {
            component.lock().unwrap().update(action.clone())?;
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, _: &State) -> Result<(), TuiError> {
        let mut a = self.buffer_rx.clone();
        let BufferAction::Count(count) = *a.borrow_and_update();
        self.progress_bar.set_progress(count.1);

        f.render_widget(Clear, rect);

        let main_component = self.components.get(self.views.last().unwrap()).unwrap();

        let chunks: std::rc::Rc<[Rect]> = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100),
                Constraint::Min(3),
                Constraint::Min(3),
            ])
            .split(Rect::new(
                rect.x + 2,
                rect.y + 1,
                rect.width.saturating_sub(6),
                rect.height.saturating_sub(0),
            ));

        //if self.state.focused == ComponentName::Search {
        //    search_block = self.make_block_focused(&self.state, search_block);
        //};
        main_component
            .lock()
            .unwrap()
            .draw(f, chunks[0], &self.state)?;
        self.components
            .get_mut(&ComponentName::Search)
            .unwrap()
            .lock()
            .unwrap()
            .draw(f, chunks[1], &self.state)?;
        self.components
            .get_mut(&ComponentName::Footer)
            .unwrap()
            .lock()
            .unwrap()
            .draw(f, chunks[2].inner(Margin::new(1, 0)), &self.state)?;

        f.render_widget(self.progress_bar.clone(), rect);
        //f.render_widget(search_block, chunks[1]);

        Ok(())
    }
}

pub fn focus_order_of(component: &ComponentName) -> Vec<ComponentName> {
    match component {
        ComponentName::RecordDetails => vec![ComponentName::RecordDetails, ComponentName::Search],
        ComponentName::Records => vec![ComponentName::Records, ComponentName::Search],
        ComponentName::TopicsAndRecords => vec![
            ComponentName::Topics,
            ComponentName::Records,
            ComponentName::Search,
        ],
        ComponentName::TopicDetails => vec![ComponentName::TopicDetails, ComponentName::Search],
        ComponentName::Help => vec![ComponentName::Help, ComponentName::Search],
        _ => vec![],
    }
}
