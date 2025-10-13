//! The footer component displays contextual information: the current cluster, shortcuts and the last notifications
use crossterm::event::KeyEvent;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Span},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, error::TuiError};

use super::{Component, ComponentName, Shortcut, State};

#[derive(Default)]
pub struct FooterComponent {
    shortcuts: Vec<Shortcut>,
    main_component: ComponentName,
    action_tx: Option<UnboundedSender<Action>>,
    show_shortcuts: bool,
}

impl FooterComponent {
    fn generate_shortcuts(&self, state: &State) -> Vec<Span<'static>> {
        let mut spans = vec![];
        for shortcut in &self.shortcuts {
            spans.push(
                format!("[{}]", shortcut.key)
                    .bold()
                    .fg(state.theme.shortcuts.unwrap_or(Color::DarkGray)),
            );
            spans.push(
                format!(":{}   ", shortcut.description)
                    .fg(state.theme.shortcuts.unwrap_or(Color::DarkGray)),
            );
        }

        spans
    }

    pub fn show_shortcuts(&mut self, visible: bool) -> &Self {
        self.show_shortcuts = visible;
        self
    }
}

impl Component for FooterComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::Footer
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<Action>, TuiError> {
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action {
            Action::Shortcuts(s, show) => {
                self.shortcuts = s;
                if show {
                    match self.main_component {
                        ComponentName::TopicsAndRecords => self
                            .shortcuts
                            .push(Shortcut::new("CTRL + O", "Hide topics")),
                        ComponentName::Records => self
                            .shortcuts
                            .push(Shortcut::new("CTRL + O", "Show topics")),
                        _ => (),
                    }
                }
                self.shortcuts.push(Shortcut::new("CTRL + H", "Help"));
                self.shortcuts.push(Shortcut::new("TAB", "Next panel"));
                self.shortcuts.push(Shortcut::new("ESC", "Quit"));
            }
            Action::ViewStack((main_component, _views)) => {
                self.main_component = main_component;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let mut help: Vec<Span<'static>> = vec![];
        if self.show_shortcuts {
            help.extend(self.generate_shortcuts(state));
        }

        let line = Line::from(help);
        f.render_widget(line, rect);
        Ok(())
    }
}

#[cfg(test)]
use crate::assert_draw;

#[test]
fn test_draw() {
    let mut component = FooterComponent::default();
    // TODO doesn't make sense
    component.show_shortcuts(true);
    component
        .update(Action::Shortcuts(
            vec![Shortcut::new("CTRL + T", "Run tests")],
            true,
        ))
        .unwrap();
    assert_draw!(component, 60, 3)
}
