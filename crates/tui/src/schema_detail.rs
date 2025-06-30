use lib::kafka::{SchemaId, SchemaRegistryClient, SchemaResponse};
use log::warn;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq, Default)]
pub struct SchemaDetail {
    pub response: Option<SchemaResponse>,
    pub url: String,
    pub id: u32,
}

impl SchemaDetail {
    pub async fn from(
        schema_registry: &mut Option<SchemaRegistryClient>,
        id: &Option<SchemaId>,
    ) -> Option<Self> {
        if id.is_none() {
            return None;
        }
        let id = id.as_ref().unwrap().0;
        let (response, url) = match schema_registry {
            Some(s) => (s.schema(id).await.ok().flatten(), s.schema_url(id)),
            None => {
                warn!(
                    "No schema registry client configured to fetch schema {id}."
                );
                (None, "".to_string())
            }
        };

        Some(Self { response, url, id })
    }
}

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq, Default)]
pub struct ExportedSchemasDetails {
    pub key: Option<SchemaDetail>,
    pub value: Option<SchemaDetail>,
}
