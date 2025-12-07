//! Work in progress

//
// ```bash
// bash docs/try-it.sh
// docker compose exec  -T schema-registry bash
// kafka-protobuf-console-producer --bootstrap-server kafka:19092 \
// --property schema.registry.url=http://localhost:8082 --topic transactions-proto \
// --property value.schema='syntax = "proto3"; message MyRecord { string id = 1; float amount = 2;}'
// { "id":"1000", "amount":500 }
// ```

// https://github.com/confluentinc/schema-registry/blob/master/protobuf-provider/src/main/java/io/confluent/kafka/schemaregistry/protobuf/ProtobufSchemaUtils.java

#[test]
fn test_protobuf_to_json() -> Result<(), ()> {
    Ok(())
}
