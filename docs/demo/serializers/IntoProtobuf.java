package serializers;
//JAVA 25+
//REPOS central,confluent=https://packages.confluent.io/maven
//DEPS com.fasterxml.jackson.core:jackson-databind:2.20.0
//DEPS com.fasterxml.jackson.dataformat:jackson-dataformat-xml:2.20.0
//DEPS org.apache.kafka:kafka-clients:4.1.0
//DEPS io.confluent:kafka-protobuf-serializer:8.0.0
//DEPS io.confluent:kafka-avro-serializer:8.0.0
//DEPS io.confluent:kafka-json-schema-serializer:8.0.0
//DEPS io.confluent:kafka-protobuf-serializer:8.0.0
//DEPS org.slf4j:slf4j-nop:2.0.16
//DEPS tech.allegro.schema.json2avro:converter:0.3.0
//DEPS com.google.protobuf:protobuf-java:4.32.1
//DEPS info.picocli:picocli:4.7.7
//DEPS org.slf4j:slf4j-api:2.0.17

//FILES protobuf/key-schema.proto=../protobuf/key-schema.proto
//FILES protobuf/value-schema.proto=../protobuf/value-schema.proto


//SOURCES Into.java
import org.apache.kafka.clients.producer.*;
import com.google.protobuf.DynamicMessage;
import io.confluent.kafka.schemaregistry.protobuf.ProtobufSchemaUtils;
import io.confluent.kafka.schemaregistry.protobuf.ProtobufSchema;


// TODO work in progress
public class IntoProtobuf implements Into<Object, Object> {
    public ProducerRecord<Object, Object> into(final String input, final String topic) throws Exception {
        var keySchemaString = readResource("/protobuf/key-schema.proto");
        var valueSchemaString = readResource("/protobuf/value-schema.proto");

        ProtobufSchema keySchema = new ProtobufSchema(keySchemaString);
        var keyString = String.format("{\"id\": \"%s\"}", this.generateKey());
        var key = (DynamicMessage) ProtobufSchemaUtils.toObject(keyString, keySchema);

        ProtobufSchema valueSchema = new ProtobufSchema(valueSchemaString);
        var value = (DynamicMessage) ProtobufSchemaUtils.toObject(input, valueSchema);

        return new ProducerRecord<>(topic, key, value);
    }
}