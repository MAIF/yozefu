//! This component renders the search bar.
//! It comes with the following features:
//!  - all queries are stored into a history.
//!  - The component suggests queries based on your history.

use std::{path::PathBuf, time::Duration};

use app::search::ValidSearchQuery;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use lib::{Error, error::SearchError};
use log::error;
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};
use tokio::{select, sync::mpsc::UnboundedSender, time::Instant};
use tokio_util::sync::CancellationToken;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    error::TuiError,
    {Action, Notification},
};

use super::{Component, ComponentName, Shortcut, State};

#[derive(Default)]
pub(crate) struct SearchComponent {
    input: Input,
    index_history: usize,
    history: Vec<String>,
    compiler_worker: CancellationToken,
    filters_directory: PathBuf,
    remaining_input: Option<String>,
    action_tx: Option<UnboundedSender<Action>>,
    autocomplete: Option<String>,
    // A hack to detect copy-paste events and replace \n with a space
    entered: Option<Instant>,
}

impl SearchComponent {
    pub fn new(input: &str, history: Vec<String>, filters_directory: PathBuf) -> Self {
        Self {
            input: Input::from(input),
            index_history: history.len().saturating_sub(1),
            history,
            filters_directory,
            ..Self::default()
        }
    }

    fn parse_input(&mut self) {
        let input = self.input.value().to_string();
        let tt = self.action_tx.clone();

        let filters_dir = self.filters_directory.clone();
        self.compiler_worker.cancel();
        self.compiler_worker = CancellationToken::new();
        let token = self.compiler_worker.clone();
        tokio::spawn(async move {
            select! {
                _ = token.cancelled() => {  },
                _ = tokio::time::sleep(Duration::from_millis(700)) => {
                    if input.len() > 5 {
                        if let Err(e) = ValidSearchQuery::from(&input, &filters_dir) {
                            error!("{e}");
                            tt.as_ref().unwrap().send(Action::Notification(Notification::new(log::Level::Error, e.to_string()))).unwrap();
                        }
                    }
                 }
            }
        });
    }

    fn autocomplete(&mut self, keycode: KeyCode) {
        let prompt = self.input.value();

        let mut possibilities = self
            .history
            .iter()
            .filter(|e| e.starts_with(prompt))
            .collect_vec();
        possibilities.dedup();

        self.index_history = match keycode {
            KeyCode::Up => match self.index_history == 0 {
                true => 0,
                false => self.index_history - 1,
            },
            KeyCode::Down => match self.index_history == possibilities.len().saturating_sub(1) {
                true => self.index_history,
                false => self.index_history + 1,
            },
            _ => match possibilities.is_empty() {
                true => 0,
                false => possibilities.len() - 1,
            },
        };
        self.autocomplete = possibilities
            .get(self.index_history)
            .map(|e| e.split_at(prompt.len()).1.to_string());
    }

    fn update_history(&mut self, prompt: &str) -> Result<(), TuiError> {
        // Do not accept empty prompts in the history
        if prompt.trim().is_empty() {
            return Ok(());
        }
        let prompt = prompt.trim().to_string();
        if !self.history.contains(&prompt) {
            self.history.push(prompt.to_string());
            self.index_history = self.history.len() - 1;
            self.action_tx
                .as_ref()
                .unwrap()
                .send(Action::NewSearchPrompt(prompt))?;
        }
        Ok(())
    }

    fn search(&mut self) -> Result<(), TuiError> {
        let o = self.input.value().to_string();

        match ValidSearchQuery::from(o.as_str(), &self.filters_directory) {
            Ok(search_query) => {
                self.update_history(&o)?;
                self.action_tx
                    .clone()
                    .unwrap()
                    .send(Action::Search(search_query))?;
            }

            Err(e) => {
                if let Error::Search(SearchError::Parse(ee)) = &e {
                    self.remaining_input = Some(ee.to_string());
                }

                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::Notification(Notification::new(
                        log::Level::Error,
                        e.to_string(),
                    )))?;
            }
        };
        Ok(())
    }

    #[allow(dead_code)]
    fn pretty_error_message(error: &nom::Err<nom::error::Error<&str>>) -> String {
        match error {
            nom::Err::Incomplete(_) => "Cannot parse query".to_string(),
            nom::Err::Error(s) => format!("unexpected token '{}'", s.input),
            nom::Err::Failure(s) => format!("unexpected token '{}'", s.input),
        }
    }
}

impl Component for SearchComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::Search
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        self.remaining_input = None;
        match key.code {
            KeyCode::Right => {
                if self.input.value().len() == self.input.cursor() {
                    if let Some(a) = &self.autocomplete {
                        let input = self.input.value().to_string();
                        let autocompleted = format!("{input}{a}");
                        self.input = self
                            .input
                            .clone()
                            .with_cursor(autocompleted.len())
                            .with_value(autocompleted);
                        self.autocomplete = None;
                        self.compiler_worker.cancel();
                    }
                }
                self.input.handle_event(&Event::Key(key));
            }
            KeyCode::Up => self.autocomplete(KeyCode::Up),
            KeyCode::Down => self.autocomplete(KeyCode::Down),
            KeyCode::Enter => {
                self.entered = Some(Instant::now());
                self.search()?;
                self.autocomplete = None;
            }
            _ if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(e) = self.entered {
                    if e.elapsed() < Duration::from_millis(10) {
                        self.input.handle_event(&Event::Key(KeyEvent::new(
                            KeyCode::Char(' '),
                            KeyModifiers::NONE,
                        )));
                    }
                    self.entered = None;
                }
                self.input.handle_event(&Event::Key(key));
                self.parse_input();
                self.autocomplete(KeyCode::Backspace);
            }
            _ => {
                self.input.handle_event(&Event::Key(key));
            }
        };
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let padding = 1;
        let input: &str = self.input.value();
        let mut line: Line = match &self.remaining_input {
            Some(e) => {
                let parts = input.split_at(input.len() - e.len());
                Line::from(vec![
                    Span::raw(parts.0),
                    Span::styled(
                        parts.1,
                        Style::default()
                            .fg(state.theme.orange)
                            .not_bold()
                            .underlined(),
                    ),
                ])
            }
            None => Line::from(vec![input.into()]),
        };

        if let Some(a) = &self.autocomplete {
            line.push_span(Span::styled(
                a,
                Style::default().fg(state.theme.autocomplete).not_bold(),
            ));
        }

        let position_cursor = Position {
            x: rect.x + 2 + (self.input.visual_cursor()) as u16,
            y: rect.y + 1,
        };

        // let rect = rect;
        //if rect.width < input.len() as u16 {
        //    let lines = input.len().div_ceil((rect.width - 15) as usize) as u16;
        //    info!("Lines {:?}", lines);
        //    rect.height =  3 + lines;
        //    rect.y = rect.y - lines;
        //    position_cursor = Position {
        //        x: rect.x + 1 + (self.input.visual_cursor()) as u16,
        //        y: rect.y + 1,
        //    }
        //}

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::left(1))
            .title(" Search ");
        let block = self.make_block_focused_with_state(state, block);

        let selected_style = Paragraph::new(line)
            .block(Block::new().padding(Padding::left(padding)))
            .wrap(Wrap { trim: false });

        let paragraph = selected_style.block(block);
        if state.is_focused(self.id()) {
            f.set_cursor_position(position_cursor);
        }
        //f.render_widget(Clear, rect);
        f.render_widget(paragraph, rect);
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![
            Shortcut::new("↑↓", "History"),
            Shortcut::new("ENTER", "Search"),
        ]
    }
}

#[cfg(test)]
use crate::assert_draw;

#[test]
fn test_draw() {
    let mut component = SearchComponent::default();
    assert_draw!(component, 60, 3)
}

#[test]
fn test_empty_history_underflow() {
    let component = SearchComponent::new("from begin", vec![], PathBuf::from("."));
    assert_eq!(component.index_history, 0);
}

//#[cfg(test)]
//use quickcheck_macros::quickcheck;
//
//#[cfg(test)]
//#[quickcheck]
//fn test_responsiveness(width: u16, height: u16) -> bool {
//    true
//    info!(
//        "Testing responsiveness with width: {}, height: {}",
//        width, height
//    );
//    use crate::Theme;
//    use app::configuration::GlobalConfig;
//    use log::info;
//    use ratatui::{Terminal, backend::TestBackend};
//    let mut component = SearchComponent::default();
//
//    let temp_dir = tempfile::tempdir().unwrap();
//    let temp_path = temp_dir.path().to_path_buf();
//
//    let state = State::new(
//        "test",
//        Theme::light(),
//        &GlobalConfig {
//            path: temp_path.clone().join("config.json"),
//            yozefu_directory: temp_path.join("config"),
//            logs: None,
//            default_url_template: "".to_string(),
//            initial_query: "".to_string(),
//            theme: "light".to_string(),
//            clusters: indexmap::IndexMap::default(),
//            default_kafka_config: indexmap::IndexMap::default(),
//            history: vec![],
//            show_shortcuts: true,
//            export_directory: std::path::PathBuf::from(""),
//        },
//    );
//
//    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
//    terminal
//        .draw(|frame| component.draw(frame, frame.area(), &state).unwrap())
//        .is_ok()
//}
