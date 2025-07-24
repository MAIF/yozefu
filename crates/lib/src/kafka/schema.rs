//! Structs and functions for key and value schemas.
#[cfg(feature = "native")]
use std::{fmt::Display, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Default)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct SchemaId(pub u32);

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub enum SchemaType {
    Json,
    Avro,
    Protobuf,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct Schema {
    pub id: SchemaId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<SchemaType>,
}

impl Schema {
    pub fn new(id: SchemaId, schema_type: Option<SchemaType>) -> Self {
        Self { id, schema_type }
    }
}

#[cfg(feature = "native")]
impl Display for SchemaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "native")]
impl Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaType::Json => write!(f, "json"),
            SchemaType::Avro => write!(f, "avro"),
            SchemaType::Protobuf => write!(f, "protobuf"),
        }
    }
}

impl SchemaId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl Default for SchemaType {
    fn default() -> Self {
        Self::Json
    }
}

#[cfg(feature = "native")]
const MAGIC_BYTE: u8 = 0;

#[cfg(feature = "native")]
impl SchemaId {
    /// More details at <https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html#wire-format>
    pub fn parse(payload: Option<&[u8]>) -> Option<Self> {
        let mut payload = payload.unwrap_or_default();
        let mut magic_byte_and_schema_id_buffer = [0u8; 5];
        match payload.read_exact(&mut magic_byte_and_schema_id_buffer) {
            Ok(_) => {
                let mut schema_id_buffer = [0u8; 4];
                if magic_byte_and_schema_id_buffer[0] != MAGIC_BYTE {
                    return None;
                }
                schema_id_buffer.copy_from_slice(&magic_byte_and_schema_id_buffer[1..]);

                Some(SchemaId(u32::from_be_bytes(schema_id_buffer)))
            }
            Err(_) => None,
        }
    }
}

#[test]
fn test_parse_schema_id() {
    assert_eq!(SchemaId::parse(None), None);
    assert_eq!(SchemaId::parse(Some(&[0, 0, 0, 0, 0])), Some(SchemaId(0)));
    assert_eq!(SchemaId::parse(Some(&[0, 0, 0, 0, 1])), Some(SchemaId(1)));
    assert_eq!(
        SchemaId::parse(Some(&[0, 0, 0, 4, 2])),
        Some(SchemaId(1026))
    );
    assert_eq!(SchemaId::parse(Some(&[54, 0, 0, 0, 1])), None);
}
