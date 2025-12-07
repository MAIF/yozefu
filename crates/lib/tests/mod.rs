pub mod deserializers;
pub mod search;

use rdkafka::message::OwnedMessage;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub struct KeyValue {
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
}

impl KeyValue {
    pub fn into_owned_message(self) -> OwnedMessage {
        OwnedMessage::new(
            self.value,
            self.key,
            "my-topic".to_string(),
            rdkafka::Timestamp::CreateTime(0),
            0,
            0,
            None,
        )
    }
}

/** Fix the timezone to a specific value for testing purposes */
pub fn fix_timezone() {
    unsafe {
        use std::env;
        // Set the timezone to Paris to have a fixed timezone for the tests
        env::set_var("TZ", "Europe/Paris");
    }
}
