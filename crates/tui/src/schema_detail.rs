use lib::kafka::{SchemaId, SchemaRegistryClient, SchemaResponse};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq, Default)]
pub struct SchemaDetail {
    pub response: Option<SchemaResponse>,
    pub url: String,
}

impl SchemaDetail {
    pub async fn from(
        schema_registry: &mut SchemaRegistryClient,
        id: &Option<SchemaId>,
    ) -> Option<Self> {
        if id.is_none() {
            return None;
        }
        let id = id.as_ref().unwrap().0;
        let url = schema_registry.schema_url(id);
        Some(Self {
            response: schema_registry.schema(id).await.ok().flatten(),
            url,
        })
    }

    pub fn schema_to_string_pretty(&self) -> Option<String> {
        self.response.as_ref().map(|r| r.schema_to_string_pretty())
    }
}

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq, Default)]
pub struct ExportedSchemasDetails {
    pub key: Option<SchemaDetail>,
    pub value: Option<SchemaDetail>,
}
