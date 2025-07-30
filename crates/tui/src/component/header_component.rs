//! The footer component displays contextual information: the current cluster, shortcuts and the last notifications
use crossterm::event::KeyEvent;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, Notification},
    error::TuiError,
};

use super::{Component, ComponentName, State};

#[derive(Default)]
pub struct HeaderComponent {
    main_component: ComponentName,
    state: Vec<ComponentName>,
    notification: Option<Notification>,
    action_tx: Option<UnboundedSender<Action>>,
    ticks: u64,
}

impl Component for HeaderComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::Header
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<Action>, TuiError> {
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action {
            Action::ViewStack((main_component, views)) => {
                self.main_component = main_component;
                self.state = views;
            }
            Action::Notification(notification) => {
                self.ticks = 0;
                self.notification = Some(notification)
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
                #[cfg(not(debug_assertions))]
                help.push("—".fg(colors.0));
                #[cfg(debug_assertions)]
                help.push("".to_string().bg(colors.0).fg(state.theme.bg));
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
            #[cfg(debug_assertions)]
            help.push("".fg(colors.0));
        }

        help.push(Span::from("  "));

        let line = Line::from(help);
        f.render_widget(line, rect);

        if let Some(n) = &self.notification {
            let notification =
                Span::styled(n.message.to_string(), Style::default().italic().not_bold());
            let r = Rect::new(
                rect.width
                    .saturating_sub(u16::try_from(n.message.len().checked_sub(1).unwrap_or(1))?),
                rect.y,
                n.message.len() as u16,
                1,
            );
            let notification = match n.level {
                tracing::Level::ERROR => notification.fg(state.theme.red).underlined(),
                tracing::Level::WARN => notification.fg(state.theme.yellow),
                _ => notification,
            };
            f.render_widget(notification, r);
        }
        Ok(())
    }
}

#[cfg(test)]
use crate::assert_draw;

#[test]
fn test_draw() {
    let mut component = HeaderComponent::default();
    assert_draw!(component, 50, 3)
}
