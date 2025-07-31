pub mod consumer_offset;

pub use consumer_offset::ConsumerOffsetKey;
pub use consumer_offset::ConsumerOffsetValue;
#[cfg(feature = "native")]
use rdkafka::message::OwnedMessage;

#[cfg(feature = "native")]
use crate::DataType;
#[cfg(feature = "native")]
use crate::kafka::schema::Schema;

#[cfg(feature = "native")]
pub(crate) fn extract_key_and_value_from_consumer_offsets_topics(
    owned_message: &OwnedMessage,
) -> (DataType, Option<Schema>, DataType, Option<Schema>) {
    use rdkafka::Message;

    let key = DataType::String(
        ConsumerOffsetKey::try_from(owned_message.key().unwrap_or_default())
            .map(|e| format!("{e:?}"))
            .or_else(|e| {
                Ok::<String, std::io::Error>(format!("Failed to parse consumer offset key: {e}"))
            })
            .unwrap(),
    );
    let value = DataType::String(
        ConsumerOffsetValue::try_from(owned_message.payload().unwrap_or_default())
            .map(|e| format!("{e:?}"))
            .or_else(|e| {
                Ok::<String, std::io::Error>(format!("Failed to parse consumer offset value: {e}"))
            })
            .unwrap(),
    );
    (key, None, value, None)
}
