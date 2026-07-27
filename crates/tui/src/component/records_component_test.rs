use crate::assert_draw;
use crate::component::Component;
use crate::component::records_component::RecordsComponent;
use lib::{DataType, KafkaRecord};

#[cfg(test)]
#[test]
fn test_draw() {
    use std::collections::BTreeMap;

    use app::configuration::DisplayOrder;
    use serde_json::json;

    use tokio::sync::mpsc::unbounded_channel;

    use crate::records_buffer::RecordsAndStats;
    let (tx, rx) = unbounded_channel();
    let mut component = RecordsComponent::new(rx, Default::default(), DisplayOrder::default());
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

#[cfg(test)]
#[test]
fn test_draw_descending_order() {
    use std::collections::BTreeMap;

    use app::configuration::DisplayOrder;
    use serde_json::json;

    use tokio::sync::mpsc::unbounded_channel;

    use crate::records_buffer::RecordsAndStats;
    let (tx, rx) = unbounded_channel();
    let mut component = RecordsComponent::new(rx, Default::default(), DisplayOrder::Descending);
    tx.send(RecordsAndStats {
        records: vec![
            KafkaRecord {
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
                "title" : "Swiss Army Man",
                "year": 2013
                }"#
                )),
                value_as_string: String::default(),
            },
            KafkaRecord {
                topic: "french-pastries".into(),
                timestamp: None,
                partition: 0,
                offset: 316,
                headers: BTreeMap::default(),
                key_schema: None,
                value_schema: None,
                size: 4348,
                key: DataType::String("french-pain-au-chocolat".into()),
                key_as_string: "french-pain-au-chocolat".into(),
                value: DataType::Json(json!(r#"{ "name" : "Pain au Chocolat" } "#)),
                value_as_string: r#"{ "name" : "Pain au Chocolat" } "#.to_string(),
            },
        ],
        read: 1,
    })
    .unwrap();

    // We should see 'pain-au-chocolat' first, since we are in descending order
    assert_draw!(component, 120, 5)
}
