use super::{Component, Shortcut, WithHeight};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{Margin, Rect},
    widgets::{Block, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, error::TuiError, tui::Event};

use super::{ComponentName, State};

#[derive(Default)]
pub(crate) struct VerticalScrollableBlock<C> {
    scroll: u16,
    scroll_length: u16,
    scrollbar_state: ScrollbarState,
    component: C,
}

impl<C> VerticalScrollableBlock<C>
where
    C: WithHeight,
{
    #[allow(dead_code)]
    pub fn new(component: C) -> Self {
        Self {
            scroll: 0,
            scroll_length: 10,
            scrollbar_state: ScrollbarState::new(component.content_height()),
            component,
        }
    }
}

impl<C> Component for VerticalScrollableBlock<C>
where
    C: WithHeight,
{
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.component.register_action_handler(tx)
    }

    fn id(&self) -> ComponentName {
        self.component.id()
    }

    fn make_block_focused_with_state<'a>(&self, state: &State, block: Block<'a>) -> Block<'a> {
        self.component.make_block_focused_with_state(state, block)
    }

    fn make_block_focused<'a>(&self, state: &State, block: Block<'a>) -> Block<'a> {
        self.component.make_block_focused(state, block)
    }

    fn init(&mut self) -> Result<(), TuiError> {
        self.component.init()
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>, TuiError> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('k') | KeyCode::Down => {
                self.scroll = (self.scroll + 1).min(self.scroll_length);
            }
            KeyCode::Char('j') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('[') => {
                self.scroll = 0;
            }
            KeyCode::Char(']') => {
                self.scroll = self.scroll_length;
            }
            _ => {
                self.component.handle_key_events(key)?;
            }
        }
        Ok(None)
    }

    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>, TuiError> {
        self.component.handle_mouse_events(mouse)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        self.component.update(action)
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        self.component.shortcuts()
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        self.scrollbar_state = self.scrollbar_state.content_length(0);
        let content_height = self.component.content_height() as u16;
        if rect.height < content_height {
            self.scroll_length = content_height - rect.height + 2;
            self.scrollbar_state = self
                .scrollbar_state
                .content_length(self.scroll_length as usize)
                .position(self.scroll as usize);
        } else {
            self.scrollbar_state = self.scrollbar_state.content_length(0);
            self.scroll = 0;
        }

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        self.component.draw(f, rect, state)?;
        f.render_stateful_widget(
            scrollbar,
            rect.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scrollbar_state,
        );
        Ok(())
    }
}
