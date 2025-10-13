use crate::component::Component;
use crate::component::records_component::RecordsComponent;
use crate::{assert_draw, component::ConcurrentRecordsBuffer, records_buffer::RecordsBuffer};
use lib::{DataType, KafkaRecord};
use std::sync::{Arc, LazyLock, Mutex};

static BUFFER: ConcurrentRecordsBuffer =
    LazyLock::new(|| Arc::new(Mutex::new(RecordsBuffer::new())));

#[cfg(test)]
#[test]
fn test_draw() {
    use std::collections::BTreeMap;

    use serde_json::json;

    let mut component = RecordsComponent::new(&BUFFER);
    BUFFER.lock().unwrap().reset();
    BUFFER.lock().unwrap().push(KafkaRecord {
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
    });

    assert_draw!(component, 120, 5)
}
