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


// jbang run ./MyProducer.java --type avro --topic public-french-addresses Nimes

package serializers;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.node.TextNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.dataformat.xml.XmlMapper;
import io.confluent.kafka.schemaregistry.json.JsonSchemaUtils;
import io.confluent.kafka.serializers.KafkaAvroSerializer;
import io.confluent.kafka.serializers.json.KafkaJsonSchemaSerializer;
import org.apache.kafka.common.serialization.ByteArraySerializer;
import org.apache.kafka.common.serialization.StringSerializer;
import io.confluent.kafka.serializers.protobuf.KafkaProtobufSerializer;
import org.apache.avro.generic.GenericData;
import org.apache.avro.Schema;
import org.apache.avro.generic.GenericRecord;
import org.apache.kafka.clients.producer.*;
import com.google.protobuf.DynamicMessage;

import java.util.*;

import io.confluent.kafka.schemaregistry.protobuf.ProtobufSchemaUtils;

import java.io.ByteArrayOutputStream;
import java.io.FileInputStream;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.concurrent.Callable;
import java.util.stream.Collectors;


//SOURCES Into.java


public class IntoText implements Into<String, String> {
    public ProducerRecord<String, String> into(final String value, final String topic) throws JsonProcessingException {
        var objectMapper = new ObjectMapper();
        var object = objectMapper.readTree(value);
        return new ProducerRecord<>(topic, this.generateKey(), object.get("properties").get("label").asText());
    }
}