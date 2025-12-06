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

//SOURCES Into.java
import java.util.*;
import java.nio.charset.StandardCharsets;
import org.apache.kafka.clients.producer.*;
import java.io.ByteArrayOutputStream;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.node.TextNode;

public class IntoMalformed implements Into<byte[], byte[]> {
    public ProducerRecord<byte[], byte[]> into(final String input, final String topic) throws Exception {
        byte randomSchemaId = (byte) ((Math.random() * (127 - 1)) + 1);
        var header = new byte[]{0, 0, 0, 0, randomSchemaId};

        ByteArrayOutputStream keyOutput = new ByteArrayOutputStream();
        keyOutput.write(header);
        keyOutput.write((generateKey() + " key").getBytes());

        randomSchemaId = (byte) ((Math.random() * (127 - 1)) + 1);
        header = new byte[]{0, 0, 0, 0, randomSchemaId};
        ByteArrayOutputStream valueOutput = new ByteArrayOutputStream();
        valueOutput.write(header);
        var objectMapper = new ObjectMapper();
        var object = objectMapper.readTree(input);
        valueOutput.write(object.get("properties").get("context").asText().getBytes(StandardCharsets.UTF_8));

        return new ProducerRecord<>(topic, keyOutput.toByteArray(), valueOutput.toByteArray());
    }
}