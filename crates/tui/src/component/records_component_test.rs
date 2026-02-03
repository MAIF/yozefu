use crate::assert_draw;
use crate::component::Component;
use crate::component::records_component::RecordsComponent;
use lib::{DataType, KafkaRecord};

#[cfg(test)]
#[test]
fn test_draw() {
    use std::collections::BTreeMap;

    use serde_json::json;

    use tokio::sync::mpsc::unbounded_channel;

    use crate::records_buffer::RecordsAndStats;
    let (tx, rx) = unbounded_channel();
    let mut component = RecordsComponent::new(rx, Default::default());
    tx.send(RecordsAndStats {
        records: vec![KafkaRecord {
            topic: "movie-trailers".into(),
            timestamp: None,
            partition: 0,
            offset: 314,
            headers: BTreeMap::default(),
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
            value_as_string: String::default(),
        }],
        read: 1,
    })
    .unwrap();

    assert_draw!(component, 120, 5)
}
