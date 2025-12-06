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

//FILES json-schema/value-schema.json=../json-schema/value-schema.json
//FILES json-schema/key-schema.json=../json-schema/key-schema.json

//SOURCES Into.java
package serializers;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.node.TextNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.util.*;
import org.apache.kafka.clients.producer.*;
import io.confluent.kafka.schemaregistry.json.JsonSchemaUtils;



public class IntoInvalidJson implements Into<JsonNode, JsonNode> {
    public ProducerRecord<JsonNode, JsonNode> into(final String input, final String topic) throws Exception {
        var objectMapper = new ObjectMapper();
        var keySchemaString = readResource("/json-schema/key-schema.json");
        var valueSchemaString = readResource("/json-schema/value-schema.json");
        var keySchema = objectMapper.readTree(keySchemaString);
        var valueSchema = objectMapper.readTree(valueSchemaString);

        var key = TextNode.valueOf(generateKey());
        var keyEnvelope = JsonSchemaUtils.envelope(keySchema, key);

        var value = objectMapper.readTree(input);
        ((ObjectNode) value).put("updatedAt", "2007");
        var valueEnvelope = JsonSchemaUtils.envelope(valueSchema, value);

        return new ProducerRecord<>(topic, keyEnvelope, valueEnvelope);
    }
}