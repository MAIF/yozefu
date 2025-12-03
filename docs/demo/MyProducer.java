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

//FILES avro/key-schema.json=avro/key-schema.json
//FILES avro/value-schema.json=avro/value-schema.json
//FILES avro/point-schema.json=avro/point-schema.json
//FILES json-schema/value-schema.json=json-schema/value-schema.json
//FILES json-schema/key-schema.json=json-schema/key-schema.json
//FILES protobuf/key-schema.proto=protobuf/key-schema.proto
//FILES protobuf/value-schema.proto=protobuf/value-schema.proto

//SOURCES serializers/Into.java
//SOURCES serializers/IntoText.java
//SOURCES serializers/IntoJson.java
//SOURCES serializers/IntoJsonSchema.java
//SOURCES serializers/IntoAvro.java
//SOURCES serializers/IntoXml.java
//SOURCES serializers/IntoProtobuf.java
//SOURCES serializers/IntoMalformed.java
//SOURCES serializers/IntoInvalidJson.java


import serializers.Into;
import serializers.IntoText;
import serializers.IntoJson;
import serializers.IntoJsonSchema;
import serializers.IntoAvro;
import serializers.IntoXml;
import serializers.IntoProtobuf;
import serializers.IntoMalformed;
import serializers.IntoInvalidJson;

// jbang run ./MyProducer.java --type avro --topic public-french-addresses Nimes

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

import io.confluent.kafka.schemaregistry.protobuf.ProtobufSchema;
import picocli.CommandLine;
import picocli.CommandLine.Command;
import serializers.IntoMalformed;
import serializers.IntoProtobuf;
import serializers.IntoText;
import serializers.IntoXml;
import tech.allegro.schema.json2avro.converter.JsonAvroConverter;


enum SerializerType {
    avro, json, jsonSchema, protobuf, text, malformed, invalidJson, xml
}

@Command(name = "MyProducer.java", version = "1.0.0", mixinStandardHelpOptions = true,
        description = "Tool to produce kafka records with different serializers."
)
class MyProducer implements Callable<Integer> {

    @CommandLine.Option(names = {"--topic"}, description = "The topic to produce records to.")
    private String topic = "public-french-addresses";

    @CommandLine.Option(names = {"--type"}, description = "avro, json, jsonSchema, protobuf, text, xml, malformed or invalidJson", defaultValue = "json")
    private SerializerType type = SerializerType.json;

    @CommandLine.Parameters(description = "Your query passed to 'https://api-adresse.data.gouv.fr/search/?q='", defaultValue = "kafka")
    private String query;

    @CommandLine.Option(names = {"--properties"}, description = "Properties file for creating the kafka producer")
    private Optional<Path> properties = Optional.empty();

    @CommandLine.Option(names = {"--every"}, description = "Produce records every X ms")
    private Optional<Long> every = Optional.empty();

    @Override
    public Integer call() throws Exception {
        Properties props = this.kafkaProperties();

        var url = System.getenv().getOrDefault("YOZEFU_API_URL", "https://api-adresse.data.gouv.fr/search/?q=%s");
        System.err.printf(" 🔩 The API is '%s'\n", url);

        System.err.printf(" 📣 About to producing records to topic '%s', serialization type is '%s'\n", topic, type);
        if(this.every.isPresent()) {
            this.produceEvery(props, url);
        } else {
            this.produceOnce(props, url);
        }

        return 0;
    }

    public Properties kafkaProperties() {
        Properties props = new Properties();
        if(this.properties.isPresent()) {
            try {
                props.load(new FileInputStream(this.properties.get().toFile()));
            } catch (IOException e) {
                e.printStackTrace();
            }
        }

        props.putIfAbsent("bootstrap.servers", "localhost:9092");
        props.putIfAbsent("schema.registry.url", System.getenv().getOrDefault("YOZEFU_SCHEMA_REGISTRY_URL", "http://localhost:8081"));
        var schemaRegistryUrl = props.getProperty("schema.registry.url");
        System.err.printf(" 📖 schema registry URL is %s\n", schemaRegistryUrl);

        return props;
    }

    public void produceOnce(Properties props, String url) throws Exception {
        var data = get(url, query);
        switch (type) {
            case avro -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, KafkaAvroSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, KafkaAvroSerializer.class.getName());
                KafkaProducer<GenericRecord, GenericRecord> producer = new KafkaProducer<>(props);
                produce(producer, new IntoAvro(), data, topic);
            }
            case json -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                KafkaProducer<String, String> producer = new KafkaProducer<>(props);
                produce(producer, new IntoJson(), data, topic);
            }
            case jsonSchema -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, KafkaJsonSchemaSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, KafkaJsonSchemaSerializer.class.getName());
                KafkaProducer<JsonNode, JsonNode> producer = new KafkaProducer<>(props);
                produce(producer, new IntoJsonSchema(), data, topic);
            }
            case protobuf -> {
                System.err.printf(" ⚠️ Protobuf serialization is experimental and may not work as expected\n");
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, KafkaProtobufSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, KafkaProtobufSerializer.class.getName());
                KafkaProducer<Object, Object> producer = new KafkaProducer<>(props);
                produce(producer, new IntoProtobuf(), data, topic);
            }
            case text -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                KafkaProducer<String, String> producer = new KafkaProducer<>(props);
                produce(producer, new IntoText(), data, topic);
            }
            case malformed -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, ByteArraySerializer.class);
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, ByteArraySerializer.class);
                KafkaProducer<byte[], byte[]> producer = new KafkaProducer<>(props);
                produce(producer, new IntoMalformed(), data, topic);
            }
            case invalidJson -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, KafkaJsonSchemaSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, KafkaJsonSchemaSerializer.class.getName());
                KafkaProducer<JsonNode, JsonNode> producer = new KafkaProducer<>(props);
                produce(producer, new IntoInvalidJson(), data, topic);
            }
            case xml -> {
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                KafkaProducer<String, String> producer = new KafkaProducer<>(props);
                produce(producer, new IntoXml(), data, topic);
            }
            default -> {
                System.err.printf(" ❕ Format '%s' is unknown. Known formats are ['avro', 'json', 'json-schema', 'text', 'malformed']\n", type);
                props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class.getName());
                KafkaProducer<String, String> producer = new KafkaProducer<>(props);
                produce(producer, new IntoText(), data, topic);
            }
        }
    }

    public void produceEvery(Properties props, String url) throws Exception {
        // run produceOnce every x ms
        var every = this.every.get();
        while(true) {
            Instant start = Instant.now();
            this.produceOnce(props, url);
            Duration timeElapsed = Duration.between(start, Instant.now()); 
            if(timeElapsed.toMillis() < every) {
                Thread.sleep(every - timeElapsed.toMillis());
            }
        }
    }

    public static <K, V> void produce(final KafkaProducer<K, V> producer, final Into<K, V> mapper, final List<String> addresses, final String topic) throws Exception {
        for (var address : addresses) {
            var record = mapper.into(address, topic);
            producer.send(record, onSend());
        }
        producer.flush();
        producer.close();
    }

    private static Callback onSend() {
        return (RecordMetadata metadata, Exception exception) -> {
            if (exception != null) {
                exception.printStackTrace();
            } else {
                System.err.println("    A new record has been produced to partition " + metadata.partition() + " with offset " + metadata.offset());
            }
        };
    }

    private static List<String> get(final String apiUrl, String query) throws IOException, InterruptedException {
        System.err.printf(" 🏡 Searching french addresses matching the query '%s'\n", query);
        var url = String.format(apiUrl, query.trim().toLowerCase());

        try(var client = HttpClient.newHttpClient()) {
            var request = HttpRequest.newBuilder()
                    .header("Accept", "application/json")
                    .uri(URI.create(url))
                    .build();
            var response = client.send(request, HttpResponse.BodyHandlers.ofString());
            var body = response.body();
            ObjectMapper mapper = new ObjectMapper();
            // System.err.println(body);
            JsonNode node = mapper.readTree(body);
            List<JsonNode> addresses = new ArrayList<>();
            if(node.isArray()) {
                for (JsonNode n : node) {
                    addresses.add(n);
                }
            }
            if(node.isObject()) {
                for (JsonNode n : node.get("features")) {
                    addresses.add(n);
                }
            }
            return addresses.stream().map(JsonNode::toString).collect(Collectors.toList());
        }
    }

    public static void main(String[] args) {
        int exitCode = new CommandLine(new MyProducer())
                .setCaseInsensitiveEnumValuesAllowed(true)
                .execute(args);
        System.exit(exitCode);
    }

}