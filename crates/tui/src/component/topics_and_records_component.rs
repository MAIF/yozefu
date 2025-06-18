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

#[cfg(test)]
use crate::assert_draw;

#[cfg(test)]
#[test]
fn test_draw() {
    use lib::{DataType, KafkaRecord};
    use serde_json::json;

    use crate::component::{
        BUFFER, records_component::RecordsComponent, topics_component::TopicsComponent,
    };

    let topics_component = TopicsComponent::new(vec!["topic1".to_string()]);
    let records_component = RecordsComponent::new(&BUFFER);
    BUFFER.lock().unwrap().reset();
    BUFFER.lock().unwrap().push(KafkaRecord {
        topic: "movie-trailers".into(),
        timestamp: None,
        partition: 0,
        offset: 314,
        headers: Default::default(),
        key_schema: None,
        value_schema: None,
        size: 4348,
        key: DataType::String("7f12bd3b-4c96-4ba1-b010-8092234eec13".into()),
        key_as_string: "7f12bd3b-4c96-4ba1-b010-8092234eec13".into(),
        value: DataType::Json(json!(
            r#"{
            {
            "title" : "Swiss Army Man",
            "year": 20013
            }
            
            }"#
        )),
        value_as_string: Default::default(),
    });

    let mut component = TopicsAndRecordsComponent::new(
        Arc::new(Mutex::new(topics_component)),
        Arc::new(Mutex::new(records_component)),
    );
    assert_draw!(component, 120, 5)
}
