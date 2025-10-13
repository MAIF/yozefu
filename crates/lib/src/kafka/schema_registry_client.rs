use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, str::FromStr, time::Duration};
use url::Url;

use crate::Error;

use super::schema::SchemaType;

#[derive(Clone, Debug)]
/// A HTTP client to communicate with a confluent schema registry
struct SimpleSchemaRegistryClient {
    url: Url,
    client: reqwest::Client,
}

impl SimpleSchemaRegistryClient {
    fn new(url: Url, headers: &HashMap<String, String>) -> Self {
        let mut default_headers = HeaderMap::new();
        // https://docs.confluent.io/platform/current/schema-registry/develop/api.html#content-types
        default_headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/vnd.schemaregistry.v1+json"),
        );
        for (key, value) in headers {
            default_headers.insert(
                HeaderName::from_str(key).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        let builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(5))
            .default_headers(default_headers);
        Self {
            url,
            client: builder.build().unwrap(),
        }
    }

    /// Tries to infer the schema type from the schema string
    fn compute_schema_type(schema: &SchemaResponse) -> Option<SchemaType> {
        match &schema.schema_type {
            Some(s) => Some(s.clone()),
            None => {
                // If the schema type is not provided, we try to infer it from the schema
                let schema_string = &schema.schema;
                match serde_json::from_str::<Value>(schema_string) {
                    Ok(v) => {
                        // is it avro ?
                        if v.get("type").is_some() && v.get("namespace").is_some() {
                            return Some(SchemaType::Avro);
                        }
                        // TODO So it should be json ?
                        // Some(SchemaType::Json)
                        None
                    }
                    Err(_) => {
                        // is it protobuf ?
                        if schema_string.contains("proto2") || schema_string.contains("proto3") {
                            return Some(SchemaType::Protobuf);
                        }
                        None
                    }
                }
            }
        }
    }

    async fn schema(&self, id: u32) -> Result<Option<SchemaResponse>, Error> {
        // TODO https://github.com/servo/rust-url/issues/333
        let url = self.schema_url(id);
        let response = self.client.get(url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let mut json = response.json::<SchemaResponse>().await.unwrap();
                    json.schema_type = Self::compute_schema_type(&json);
                    return Ok(Some(json));
                }
                Ok(None)
            }

            Err(e) => Err(Error::SchemaRegistry(e.to_string())),
        }
    }

    fn schema_url(&self, id: u32) -> String {
        // TODO https://github.com/servo/rust-url/issues/333
        let mut url = self.url.clone();
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.extend(vec!["schemas", "ids", &id.to_string()]);
        }
        url.to_string()
    }
}

#[derive(Clone, Debug)]
/// A HTTP client to communicate with a confluent schema registry
/// All schemas are cached
pub struct SchemaRegistryClient {
    client: SimpleSchemaRegistryClient,
    cache: HashMap<u32, SchemaResponse>,
}

impl SchemaRegistryClient {
    pub fn new(base_url: Url, headers: &HashMap<String, String>) -> Self {
        Self {
            client: SimpleSchemaRegistryClient::new(base_url, headers),
            cache: HashMap::default(),
        }
    }

    pub async fn schema(&mut self, id: u32) -> Result<Option<SchemaResponse>, Error> {
        match self.cache.get(&id) {
            Some(schema) => Ok(Some(schema.clone())),
            None => {
                let schema = self.client.schema(id).await?;
                if let Some(schema) = schema.clone() {
                    self.cache.insert(id, schema.clone());
                }
                Ok(schema)
            }
        }
    }

    pub fn schema_url(&self, id: u32) -> String {
        self.client.schema_url(id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaResponse {
    pub schema: String,
    pub schema_type: Option<SchemaType>,
}

impl SchemaResponse {
    pub fn schema_to_string_pretty(&self) -> String {
        match self.schema_type {
            Some(SchemaType::Avro | SchemaType::Json) => {
                let json = serde_json::from_str::<Value>(&self.schema)
                    .unwrap_or(Value::String(String::new()));
                serde_json::to_string_pretty(&json).unwrap()
            }
            _ => self.schema.clone(),
        }
    }
}
