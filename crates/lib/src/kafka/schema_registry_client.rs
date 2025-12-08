use reqwest::{
    Response,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
    time::Duration,
};
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

    async fn schema(&self, id: u32) -> Result<Option<MessageSchema>, Error> {
        // TODO https://github.com/servo/rust-url/issues/333
        let url = self.schema_url(id);
        let response = self.client.get(&url).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    return self.response_to_schema(response).await;
                }
                Ok(None)
            }
            Err(e) => Err(Error::SchemaRegistry(e.to_string())),
        }
    }

    async fn response_to_schema(&self, response: Response) -> Result<Option<MessageSchema>, Error> {
        let mut json = response.json::<SchemaResponse>().await.unwrap();
        json.schema_type = Self::compute_schema_type(&json);

        let mut schemas = vec![json.schema];
        if let Some(referenced_subjects) = json.references.take() {
            if let Some(ref_schemas) = self.referenced_subjects(referenced_subjects).await? {
                schemas.extend(ref_schemas);
            }
        }
        Ok(Some(MessageSchema {
            schemas,
            schema_type: json.schema_type,
        }))
    }

    async fn referenced_subjects(
        &self,
        referenced_subjects: Vec<SchemaReference>,
    ) -> Result<Option<Vec<String>>, Error> {
        //Avro-rs does not like multiples of the same schema
        let mut seen: HashSet<String> = HashSet::new();
        let mut schemas = vec![];

        let mut ref_subjects_to_fetch = VecDeque::from(referenced_subjects);
        while let Some(subject) = ref_subjects_to_fetch.pop_front() {
            if !seen.contains(&subject.subject) {
                let subject_response = self.subject_schema(&subject).await;
                seen.insert(subject.subject);
                match subject_response {
                    Ok(Some(subject_response)) => {
                        if let Some(mut references) = subject_response.references {
                            ref_subjects_to_fetch.extend(references.drain(..));
                        }
                        schemas.push(subject_response.schema);
                    }
                    Ok(None) => return Ok(None),
                    Err(e) => return Err(Error::SchemaRegistry(e.to_string())),
                };
            }
        }
        Ok(Some(schemas))
    }

    async fn subject_schema(
        &self,
        referenced: &SchemaReference,
    ) -> Result<Option<SchemaResponse>, Error> {
        let url = self.subject_schema_url(&referenced.subject, referenced.version);
        let response = self.client.get(url.clone()).send().await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let schema_response = response.json::<SchemaResponse>().await.unwrap();
                    return Ok(Some(schema_response));
                }
                Ok(None)
            }
            Err(e) => Err(Error::SchemaRegistry(e.to_string())),
        }
    }

    fn subject_schema_url(&self, subject_name: &str, id: u16) -> String {
        // TODO https://github.com/servo/rust-url/issues/333
        let mut url = self.url.clone();
        if let Ok(mut segments) = url.path_segments_mut() {
            segments.extend(vec!["subjects", subject_name, "versions", &id.to_string()]);
        }
        url.to_string()
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
    cache: HashMap<u32, MessageSchema>,
}

impl SchemaRegistryClient {
    pub fn new(base_url: Url, headers: &HashMap<String, String>) -> Self {
        Self {
            client: SimpleSchemaRegistryClient::new(base_url, headers),
            cache: HashMap::default(),
        }
    }

    pub async fn schema(&mut self, id: u32) -> Result<Option<MessageSchema>, Error> {
        match self.cache.get(&id) {
            Some(schema) => Ok(Some(schema.clone())),
            None => {
                let schema = self.client.schema(id).await?;
                if let Some(schema) = &schema {
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
pub struct MessageSchema {
    pub schemas: Vec<String>,
    pub schema_type: Option<SchemaType>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SchemaResponse {
    pub schema: String,
    pub references: Option<Vec<SchemaReference>>,
    pub schema_type: Option<SchemaType>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SchemaReference {
    pub subject: String,
    pub version: u16,
}

impl MessageSchema {
    pub fn schema_to_string_pretty(&self) -> String {
        match self.schema_type {
            Some(SchemaType::Avro | SchemaType::Json) => {
                let json = serde_json::from_str::<Value>(self.schemas.first().unwrap())
                    .unwrap_or(Value::String(String::new()));
                serde_json::to_string_pretty(&json).unwrap()
            }
            _ => self.schemas.first().unwrap().clone(),
        }
    }
}
