#[cfg(feature = "native")]
use super::SchemaRegistryClient;
#[cfg(feature = "native")]
use super::avro::avro_to_json;
use super::data_type::DataType;
use super::schema::Schema;
#[cfg(feature = "native")]
use super::schema::SchemaId;
#[cfg(feature = "native")]
use super::schema::SchemaType;
#[cfg(feature = "native")]
use super::schema_registry_client::SchemaResponse;
#[cfg(feature = "native")]
use crate::kafka::internal::extract_key_and_value_from_consumer_offsets_topics;
#[cfg(feature = "native")]
use apache_avro::from_avro_datum;
#[cfg(feature = "native")]
use chrono::{DateTime, Local, Utc};
#[cfg(feature = "native")]
use rdkafka::message::{Headers, Message, OwnedMessage};
use serde::Deserialize;
use serde::Serialize;
#[cfg(feature = "native")]
use serde_json::Error;
use std::collections::BTreeMap;

/// Inspired of the `[rdkafka::Message]` struct.
/// Currently, we only support utf-8 string keys/values/headers.
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct KafkaRecord {
    pub topic: String,
    pub timestamp: Option<i64>,
    pub partition: i32,
    pub offset: i64,
    pub headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_schema: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_schema: Option<Schema>,
    /// Number of bytes for the key + the value
    #[serde(default)]
    pub size: usize,
    /// A human-readable representation of the key
    pub key: DataType,
    #[serde(skip_serializing, default)]
    pub key_as_string: String,
    /// A human-readable representation of the value
    pub value: DataType,
    #[serde(skip_serializing, default)]
    /// The value as a string. needed to be displayed in the TUI
    pub value_as_string: String,
}

#[cfg(feature = "native")]
impl KafkaRecord {
    pub fn timestamp_as_utc_date_time(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(self.timestamp.unwrap_or(0))
    }

    pub fn timestamp_as_local_date_time(&self) -> Option<DateTime<Local>> {
        self.timestamp_as_utc_date_time()
            .map(DateTime::<Local>::from)
    }

    pub fn has_schemas(&self) -> bool {
        self.key_schema.is_some() || self.value_schema.is_some()
    }
}

#[cfg(feature = "native")]
impl KafkaRecord {
    pub async fn parse(
        owned_message: OwnedMessage,
        schema_registry: &mut Option<SchemaRegistryClient>,
    ) -> Self {
        let mut headers: BTreeMap<String, String> = BTreeMap::new();
        if let Some(old_headers) = owned_message.headers() {
            for header in old_headers.iter() {
                headers.insert(
                    header.key.to_string(),
                    header
                        .value
                        .map(|e| {
                            String::from_utf8(e.to_vec()).unwrap_or("<unable to parse>".to_string())
                        })
                        .unwrap_or_default(),
                );
            }
        }

        let size = owned_message.payload().map_or(0, <[u8]>::len)
            + owned_message.key().map_or(0, <[u8]>::len);

        let (key, key_schema, value, value_schema) = match owned_message.topic() {
            "__consumer_offsets" => {
                extract_key_and_value_from_consumer_offsets_topics(&owned_message)
            }
            _ => {
                let (key, key_schema) =
                    Self::extract_data_and_schema(owned_message.key(), schema_registry).await;
                let (value, value_schema) =
                    Self::extract_data_and_schema(owned_message.payload(), schema_registry).await;
                (key, key_schema, value, value_schema)
            }
        };

        Self {
            value_as_string: value.to_string(),
            value,
            key_as_string: key.to_string(),
            key,
            topic: owned_message.topic().to_string(),
            timestamp: owned_message.timestamp().to_millis(),
            partition: owned_message.partition(),
            offset: owned_message.offset(),
            headers,
            key_schema,
            value_schema,
            size,
        }
    }

    fn payload_to_data_type(payload: Option<&[u8]>, schema: Option<&SchemaResponse>) -> DataType {
        if schema.is_none() {
            return Self::deserialize_json(payload);
        }

        let schema = schema.as_ref().unwrap();
        match schema.schema_type {
            Some(SchemaType::Json) => Self::deserialize_json(payload),
            Some(SchemaType::Avro) => Self::deserialize_avro(payload, &schema.schema),
            Some(SchemaType::Protobuf) => Self::deserialize_protobuf(payload, &schema.schema),
            None => Self::deserialize_json(payload),
        }
    }

    /// Fallback to String if this is not json
    /// Will I regret it ? Maybe
    fn try_deserialize_json(payload: Option<&[u8]>) -> Result<DataType, Error> {
        let payload = payload.unwrap_or_default();
        match serde_json::from_slice(payload) {
            Ok(e) => Ok(DataType::Json(e)),
            Err(e) => Err(e),
        }
    }

    /// Fallback to String if this is not json
    /// Will I regret it ? Maybe
    fn deserialize_json(payload: Option<&[u8]>) -> DataType {
        match Self::try_deserialize_json(payload) {
            Ok(e) => e,
            Err(_e) => DataType::String(
                String::from_utf8(payload.unwrap_or_default().to_vec()).unwrap_or_default(),
            ),
        }
    }

    fn deserialize_avro(payload: Option<&[u8]>, schema: &str) -> DataType {
        let mut payload = payload.unwrap_or_default();
        let parsed_schema = apache_avro::Schema::parse_str(schema);
        if let Err(e) = &parsed_schema {
            return DataType::String(format!(
                "  Yozefu Error: The avro schema could not be parsed. Please check the schema in the schema registry.\n       Error: {}\n       Payload: {:?}\n        String: {}",
                e,
                payload,
                String::from_utf8(payload.to_vec()).unwrap_or_default()
            ));
        }
        let parsed_schema = parsed_schema.unwrap();
        match from_avro_datum(&parsed_schema, &mut payload, None) {
            Ok(value) => DataType::Json(avro_to_json(value)),
            Err(e) => DataType::String(format!(
                "  Yozefu Error: According to the schema registry, the record is serialized as avro but there was an issue deserializing the payload: {:?}\n       Payload: {:?}\n        String: {}",
                e,
                payload,
                String::from_utf8(payload.to_vec()).unwrap_or_default()
            )),
        }
    }

    fn deserialize_protobuf(payload: Option<&[u8]>, schema: &str) -> DataType {
        let payload = payload.unwrap_or_default();
        DataType::String(format!(
            "  Error: Protobuf deserialization is not supported yet in Yozefu. Any contribution is welcome!\n Github: https://github.com/MAIF/yozefu\nPayload: {:?}\n String: {}\n Schema:\n{}",
            payload,
            String::from_utf8(payload.to_vec())
                .unwrap_or_default()
                .trim(),
            schema,
        ))
    }

    /// Extract the data section from the payload prefixed with a schema section.
    fn extract_data_from_payload_with_schema_header(payload: &[u8]) -> Option<&[u8]> {
        if payload.len() <= 5 {
            return None;
        }
        Some(&payload[5..])
    }

    async fn extract_data_and_schema(
        payload: Option<&[u8]>,
        schema_registry: &mut Option<SchemaRegistryClient>,
    ) -> (DataType, Option<Schema>) {
        let schema_id = SchemaId::parse(payload);
        match (schema_id, schema_registry.as_mut()) {
            (None, _) => (Self::payload_to_data_type(payload, None), None),
            (Some(id), None) => {
                let payload = payload.unwrap_or_default();
                match serde_json::from_slice(payload) {
                    Ok(e) => (DataType::Json(e), None),
                    Err(_e) => {
                        match Self::try_deserialize_json(
                            Self::extract_data_from_payload_with_schema_header(payload),
                        ) {
                            Ok(e) => (e, Some(Schema::new(id, None))),
                            Err(_e) => (
                                DataType::String(format!(
                                    "Yozefu was not able to retrieve the schema {} because there is no schema registry configured. Please visit https://maif.github.io/yozefu/schema-registry/ for more details.\nPayload: {:?}\n String: {}",
                                    id,
                                    payload,
                                    String::from_utf8(payload.to_vec()).unwrap_or_default()
                                )),
                                Some(Schema::new(id, None)),
                            ),
                        }
                    }
                }
            }
            (Some(s), Some(schema_registry)) => {
                let p = payload.unwrap_or_default();
                let (schema_response, schema) = match schema_registry.schema(s.0).await {
                    Ok(Some(d)) => (Some(d.clone()), Some(Schema::new(s, d.schema_type))),
                    Ok(None) => (None, Some(Schema::new(s, None))),
                    Err(e) => {
                        let payload = payload.unwrap_or_default();
                        return (
                            DataType::String(format!(
                                "{}.\nYozefu was not able to retrieve the schema {}.\nPlease make sure the schema registry is correctly configured.\nPayload: {:?}\n String: {}",
                                e,
                                s.0,
                                payload,
                                String::from_utf8(payload.to_vec()).unwrap_or_default()
                            )),
                            Some(Schema::new(s, None)),
                        );
                    }
                };
                match p.len() <= 5 {
                    true => (
                        Self::payload_to_data_type(payload, schema_response.as_ref()),
                        schema,
                    ),
                    false => (
                        Self::payload_to_data_type(
                            payload.map(|e| e[5..].as_ref()),
                            schema_response.as_ref(),
                        ),
                        schema,
                    ),
                }
            }
        }
    }
}

#[test]
fn test_payload_to_data_type() {
    let d = KafkaRecord::payload_to_data_type(Some("true".as_bytes()), None);
    assert_eq!(d, DataType::Json(serde_json::json!(true)));
}
