//! Component listing all the kafa topics that can be consumed
use std::collections::HashSet;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use ratatui::prelude::Stylize;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::component::topics_list::TopicList;
use crate::{Action, error::TuiError};

use super::{Component, ComponentName, Shortcut, State};

#[derive(Default)]
pub(crate) struct TopicsComponent {
    topics: TopicList,
    state: ListState,
    action_tx: Option<UnboundedSender<Action>>,
    input: Input,
    loading: bool,
}

impl TopicsComponent {
    pub fn new(selected_topics: Vec<String>) -> TopicsComponent {
        let loading = selected_topics.is_empty();
        let topics = selected_topics.clone();
        let index = if loading { None } else { Some(1) };
        Self {
            topics: TopicList::new(topics.clone(), selected_topics.clone()),
            state: ListState::default().with_selected(index),
            loading,
            ..Default::default()
        }
    }

    pub fn topics(&self) -> &[String] {
        self.topics.all()
    }

    fn next(&mut self) {
        if self.topics.get().is_empty() {
            return;
        }

        match self.state.selected() {
            Some(i) => {
                if i >= self.topics.get().len() - 1 {
                    self.state.select(Some(0));
                } else {
                    self.state.select(Some(i + 1));
                }
            }
            None => self.state.select(Some(0)),
        };
    }

    fn previous(&mut self) {
        let topics = self.topics.get();
        if topics.is_empty() {
            return;
        }
        match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.state.select(Some(topics.len() - 1));
                } else {
                    self.state.select(Some(i - 1));
                }
            }
            None => self.state.select(Some(0)),
        };
    }

    fn filter_topics(&mut self) {
        self.topics.set_filter(self.input.value().trim());
        if self.topics.get().is_empty() {
            self.state.select(Some(0));
        }
    }
}

impl Component for TopicsComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::Topics
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(selected) = self.state.selected() {
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::NewView(ComponentName::TopicDetails))?;

                    let mut h = HashSet::default();
                    h.insert(self.topics.get().get(selected).unwrap().to_string());
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::RequestTopicDetails(h))?;
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.topics.clear_selected();
                self.action_tx
                    .clone()
                    .unwrap()
                    .send(Action::RefreshShortcuts)?;

                self.filter_topics();
                self.action_tx
                    .clone()
                    .unwrap()
                    .send(Action::SelectedTopics(vec![]))?;
            }
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            KeyCode::Enter => {
                if self.state.selected().is_none() {
                    return Ok(None);
                }

                let topic = self
                    .topics
                    .get()
                    .get(self.state.selected().unwrap())
                    .cloned();
                if topic.is_none() {
                    return Ok(None);
                }
                let topic = topic.unwrap().to_string();
                self.topics.toggle_topics(&topic);

                self.action_tx
                    .clone()
                    .unwrap()
                    .send(Action::SelectedTopics(
                        self.topics.selected().iter().cloned().collect_vec(),
                    ))?;
                self.action_tx
                    .clone()
                    .unwrap()
                    .send(Action::RefreshShortcuts)?;
            }
            KeyCode::Esc => (),
            _ => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.input.handle_event(&Event::Key(key));
                    self.filter_topics();
                }
            }
        };
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        if let Action::Topics(new_topics) = action {
            self.topics.refresh_topics(new_topics);
            self.loading = false;
            if !self.topics.get().is_empty() {
                let selected = self.state.selected().unwrap_or(0);
                match selected < self.topics.get().len() {
                    true => self.state.select(Some(selected)),
                    false => self.state.select(Some(0)),
                }
            }
        };
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let is_focused = state.is_focused(self.id());
        let title = match self.topics.selected().len() {
            0 => " Topics ".to_string(),
            selected => format!(" Topics [{selected}] "),
        };
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title);
        let outer_block = self.make_block_focused_with_state(state, outer_block);

        let items: Vec<ListItem> = self
            .topics
            .get_with_selection()
            .iter()
            .map(|t| match t.1 {
                true => ListItem::new(format!(" [x] {} ", t.0)).style(Style::default().bold()),
                false => ListItem::new(format!(" [ ] {} ", t.0)).style(Style::default()),
            })
            .collect();

        let list = List::new(items).highlight_style(match is_focused {
            true => Style::default()
                .bg(state.theme.bg_focused_selected)
                .fg(state.theme.fg_focused_selected)
                .bold(),
            false => Style::default()
                .bg(state.theme.bg_unfocused_selected)
                .fg(state.theme.fg_unfocused_selected),
        });

        let mut filter_block = Block::default()
            .title(" Search ")
            .padding(Padding::left(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default());

        if is_focused {
            filter_block = filter_block
                .border_type(BorderType::Thick)
                .title_style(Style::default().bold())
                .border_style(Style::default().fg(state.theme.dialog_border));
        }

        let filter = Paragraph::new(self.input.value())
            .style(Style::default())
            .block(filter_block);

        let inner = outer_block.inner(rect);
        f.render_widget(outer_block, rect);

        match self.input.value().is_empty() {
            true => f.render_stateful_widget(list, inner, &mut self.state),
            false => {
                let filter = filter.style(Style::default());
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100), Constraint::Min(3)])
                    .split(inner);
                if is_focused {
                    f.set_cursor_position(Position {
                        x: layout[1].x + (self.input.visual_cursor()) as u16 + 2,
                        y: layout[1].y + 1,
                    });
                }
                f.render_stateful_widget(list, layout[0], &mut self.state);
                f.render_widget(Clear, layout[1]);
                f.render_widget(filter, layout[1]);
            }
        }

        if self.loading {
            let loading = Paragraph::new("[/] Loading topics...").style(Style::default());
            f.render_widget(loading, inner);
        }

        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        let mut shortcuts = vec![
            Shortcut::new("ENTER", "Consume topic"),
            Shortcut::new("CTRL + P", "Show details"),
        ];

        if !self.topics.any_selected() {
            shortcuts.push(Shortcut::new("CTRL + U", "Unselect topics"));
        }
        shortcuts
    }
}

#[cfg(test)]
use crate::assert_draw;

#[test]
fn test_draw() {
    let mut component = TopicsComponent::default();
    component
        .update(Action::Topics(vec![
            "private-random-people".to_string(),
            "public-french-addresses".to_string(),
            "public-patisserie-delights".to_string(),
        ]))
        .unwrap();
    assert_draw!(component, 60, 5)
}
