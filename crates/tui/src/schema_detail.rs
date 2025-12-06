use lib::kafka::{Schema, SchemaId, SchemaRegistryClient};
use serde::Serialize;
use tracing::warn;

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq, Default)]
pub struct SchemaDetail {
    pub response: Option<Schema>,
    pub url: String,
    pub id: u32,
}

impl SchemaDetail {
    pub async fn from(
        schema_registry: &mut Option<SchemaRegistryClient>,
        id: Option<&SchemaId>,
    ) -> Option<Self> {
        let id = id.as_ref()?.0;
        let (response, url) = match schema_registry {
            Some(s) => (s.schema(id).await.ok().flatten(), s.schema_url(id)),
            None => {
                warn!("No schema registry client configured to fetch schema {id}.");
                (None, String::new())
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
