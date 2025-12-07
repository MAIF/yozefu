use insta::assert_json_snapshot;
use std::{collections::HashMap, fs, path::PathBuf};
use url::Url;
use yozefu_lib::{KafkaRecord, kafka::SchemaRegistryClient};

use crate::{KeyValue, fix_timezone};

/// Returns the current directory of the test files.
fn current_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("deserializers")
        .join("avro")
}

/// Macro to create a mock schema registry server with predefined schema responses.
macro_rules! mock_schema_registry {
    ({ $($id:literal => $path:expr),+ $(,)? }) => {{
        let mut server = mockito::Server::new_async().await;
        $(
            server
                .mock("GET", concat!("/schemas/ids/", stringify!($id)))
                .with_status(200)
                .match_header("accept", mockito::Matcher::Any)
                .with_body(include_str!($path))
                .create_async().await;
        )+
        let client = SchemaRegistryClient::new(
            Url::parse(&server.url()).unwrap(),
            &HashMap::default()
        );
        (server, client)
    }};
}

#[tokio::test]
/// Test deserialization of an Avro record, key and value schemas are fetched from the mock schema registry.
async fn test_avro_record() {
    fix_timezone();
    let input = fs::read_to_string(current_directory().join("inputs/records/record.json")).unwrap();
    let owned_message = serde_json::from_str::<KeyValue>(&input)
        .unwrap()
        .into_owned_message();

    let (_server, schema_client) = mock_schema_registry! {{
        1 => "./inputs/schemas/key.json",
        2 => "./inputs/schemas/value.json"
    }};

    let record = KafkaRecord::parse(owned_message, &mut Some(schema_client)).await;
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(record);
    });
}

#[tokio::test]
/// Test deserialization of an Avro record, one of the primitive types in the schema is unknown.
async fn test_avro_record_unknown_primitive_type() {
    fix_timezone();
    let input =
        fs::read_to_string(current_directory().join("inputs/records/record-schema-reference.json"))
            .unwrap();
    let key_value: KeyValue = serde_json::from_str(&input).unwrap();
    let owned_message = key_value.into_owned_message();

    let (_server, schema_client) = mock_schema_registry! {{
        1 => "./inputs/schemas/key.json",
        3 => "./inputs/schemas/value-with-reference.json"
    }};
    let record = KafkaRecord::parse(owned_message, &mut Some(schema_client)).await;
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(record);
    });
}

#[tokio::test]
/// Test deserialization of an Avro record with the value schema referencing another schema in the schema registry.
async fn test_avro_record_with_schema_reference() {
    fix_timezone();
    let input =
        fs::read_to_string(current_directory().join("inputs/records/record-schema-reference.json"))
            .unwrap();
    let key_value: KeyValue = serde_json::from_str(&input).unwrap();
    let owned_message = key_value.into_owned_message();

    let (_server, schema_client) = mock_schema_registry! {{
        1 => "./inputs/schemas/key.json",
        2 => "./inputs/schemas/schema-reference.json",
        3 => "./inputs/schemas/value-with-reference.json"
    }};

    let record = KafkaRecord::parse(owned_message, &mut Some(schema_client)).await;
    insta::with_settings!({sort_maps => true}, {
        assert_json_snapshot!(record);
    });
}
