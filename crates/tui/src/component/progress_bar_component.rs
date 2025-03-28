//! Progress bar at the top of the window when you consume kafka records.
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

#[derive(Clone, Default)]
pub(crate) struct ProgressBarComponent {
    length: u64,
    progress: u64,
}

impl ProgressBarComponent {
    pub fn new(length: u64) -> Self {
        Self {
            length,
            progress: Default::default(),
        }
    }

    pub fn set_progress(&mut self, inc: usize) {
        self.progress = inc as u64;
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length as u64;
    }
}

impl Widget for ProgressBarComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.progress == 0 || self.length == 0 {
            return;
        }
        let percent = 100 * self.progress / self.length;
        let dd = (area.right() - area.left()) as u64 * percent / 100;
        buf.set_string(
            area.left(),
            area.top(),
            (0..dd).map(|_| "▔").collect::<String>(),
            Style::default().fg(Color::Green),
        );
    }
}
