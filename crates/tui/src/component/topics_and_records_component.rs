//! This component is a layout component that renders `[TopicsComponent]` and `[RecordsComponent]`.
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Clear,
};
use std::sync::{Arc, Mutex};

use crate::error::TuiError;

use super::{Component, ComponentName, State};

pub(crate) struct TopicsAndRecordsComponent {
    records: Arc<Mutex<dyn Component>>,
    topics: Arc<Mutex<dyn Component>>,
}

impl TopicsAndRecordsComponent {
    pub fn new(topics: Arc<Mutex<dyn Component>>, records: Arc<Mutex<dyn Component>>) -> Self {
        Self { records, topics }
    }
}

impl Component for TopicsAndRecordsComponent {
    fn id(&self) -> ComponentName {
        ComponentName::TopicsAndRecords
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        f.render_widget(Clear, rect);

        let chunks: std::rc::Rc<[Rect]> = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(62), Constraint::Percentage(100)])
            .spacing(1)
            .split(rect);

        self.topics.lock().unwrap().draw(f, chunks[0], state)?;
        self.records.lock().unwrap().draw(f, chunks[1], state)?;
        Ok(())
    }
}
