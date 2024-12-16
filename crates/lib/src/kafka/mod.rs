#[cfg(feature = "native")]
pub mod exported_kafka_record;
#[cfg(feature = "native")]
pub use exported_kafka_record::ExportedKafkaRecord;
#[cfg(feature = "native")]
mod schema_registry_client;
#[cfg(feature = "native")]
pub mod topic;
#[cfg(feature = "native")]
pub use schema_registry_client::SchemaRegistryClient;
#[cfg(feature = "native")]
mod avro;

mod data_type;
mod kafka_record;
mod schema;
pub use data_type::Comparable;
pub use data_type::DataType;
pub use kafka_record::KafkaRecord;
pub use schema::SchemaId;
pub use schema_registry_client::SchemaResponse;
