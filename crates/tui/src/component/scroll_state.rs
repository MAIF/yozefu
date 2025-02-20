//! Scroll state for the widgets with vertical scroll

use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState as RatatuiScrollbarState},
};

#[derive(Default)]
pub(crate) struct ScrollState {
    rect: Rect,
    scroll: usize,
    scroll_height: usize,
    scrollbar_state: RatatuiScrollbarState,
}

impl ScrollState {
    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
        self.update_position();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.scroll_height;
        self.update_position();
    }

    fn update_position(&mut self) {
        self.scrollbar_state = RatatuiScrollbarState::new(self.scroll_height).position(self.scroll);
    }

    /// Set height of the scrollbar
    pub fn set_height(&mut self, height: usize) {
        self.scroll_height = height.saturating_sub(self.rect.height as usize);
        self.scroll = self.scroll_height.min(self.scroll);
        self.update_position();
    }

    pub fn scroll_to_next_line(&mut self) {
        if self.scroll < self.scroll_height {
            self.scroll += 1;
            self.scrollbar_state = self.scrollbar_state.position(self.scroll);
        }
        self.update_position();
    }

    /// Position of the scroll
    pub fn value(&mut self) -> u16 {
        self.scroll as u16
    }

    /// Reset scroll to 0
    pub fn reset(&mut self) {
        self.scroll = 0;
        self.update_position();
    }

    pub fn scroll_to_previous_line(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
            self.scrollbar_state = self.scrollbar_state.position(self.scroll);
        }
    }

    pub fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, scroll_height: usize) {
        if self.rect != rect {
            self.rect = rect;
            //info!(
            //    "scroll state rect: {:?}, height: {}",
            //    self.rect, self.scroll_height
            //);
        }

        self.set_height(scroll_height);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));

        f.render_stateful_widget(scrollbar, self.rect, &mut self.scrollbar_state);
    }
}
