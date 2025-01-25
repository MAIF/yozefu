//! The footer component displays contextual information: the current cluster, shortcuts and the last notifications
use crossterm::event::KeyEvent;

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Notification},
    error::TuiError,
};

use super::{Component, ComponentName, Shortcut, State};

#[derive(Default)]
pub struct FooterComponent {
    shortcuts: Vec<Shortcut>,
    main_component: ComponentName,
    state: Vec<ComponentName>,
    notification: Option<Notification>,
    action_tx: Option<UnboundedSender<Action>>,
    ticks: u64,
    show_shortcuts: bool,
}

impl FooterComponent {
    fn generate_shortcuts(&self, _state: &State) -> Vec<Span<'static>> {
        let mut spans = vec![];
        for shortcut in &self.shortcuts {
            spans.push(format!("[{}]", shortcut.key).bold());
            spans.push(format!(":{}   ", shortcut.description).into());
        }

        spans
    }

    pub fn show_shortcuts(&mut self, visible: bool) -> &Self {
        self.show_shortcuts = visible;
        self
    }
}

impl Component for FooterComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), TuiError> {
        self.action_tx = Some(tx);
        Ok(())
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
                    };
                }
                self.shortcuts.push(Shortcut::new("CTRL + H", "Help"));
                self.shortcuts.push(Shortcut::new("TAB", "Next panel"));
                self.shortcuts.push(Shortcut::new("ESC", "Quit"));
            }
            Action::ViewStack((main_component, views)) => {
                self.main_component = main_component;
                self.state = views;
            }
            Action::Notification(notification) => {
                self.ticks = 0;
                self.notification = Some(notification)
            }
            Action::ResetNotification() => {
                self.notification = None;
                self.ticks = 0;
            }
            Action::Tick => {
                self.ticks += 1;
                if self.ticks > 20 {
                    self.notification = None;
                }
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let mut view_stack = self.state.clone();
        view_stack.dedup();
        view_stack.push(state.focused.clone());
        let mut help = vec![];
        help.push(
            format!(" {} ", state.cluster)
                .black()
                .bold()
                .bg(state.theme.white),
        );
        help.push(" ".into());
        for v in view_stack.iter().enumerate() {
            let colors = match v.0 == view_stack.len() - 1 {
                true => (state.theme.bg_active, state.theme.fg_active),
                false => (state.theme.bg_disabled, state.theme.fg_disabled),
            };
            if v.0 > 0 {
                help.push("—".fg(colors.0));
            }
            let prefix = match v.0 {
                0 if self.main_component == ComponentName::TopicsAndRecords => "◧ ",
                0 if self.main_component == ComponentName::Records => "□ ",
                _ => "",
            };

            help.push(
                format!(" {}{:<8}", prefix, v.1.label())
                    .bg(colors.0)
                    .fg(colors.1)
                    .bold(),
            );
        }

        help.push(Span::from("  "));
        if self.show_shortcuts {
            help.extend(self.generate_shortcuts(state));
        }

        let line = Line::from(help);
        f.render_widget(line, rect);

        if let Some(n) = &self.notification {
            let notification =
                Span::styled(n.message.to_string(), Style::default().italic().not_bold());
            let r = Rect::new(
                rect.width
                    .saturating_sub(u16::try_from(n.message.len().checked_sub(3).unwrap_or(3))?),
                rect.y,
                n.message.len() as u16,
                1,
            );
            let notification = match n.level {
                log::Level::Error => notification.fg(Color::LightRed).underlined(),
                log::Level::Warn => notification.fg(Color::Yellow),
                _ => notification,
            };
            f.render_widget(notification, r);
        }
        Ok(())
    }
}
