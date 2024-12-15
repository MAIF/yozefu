//JAVA 21+
//REPOS central,confluent=https://packages.confluent.io/maven
//DEPS com.fasterxml.jackson.core:jackson-databind:2.18.2
//DEPS com.fasterxml.jackson.dataformat:jackson-dataformat-xml:2.18.2
//DEPS org.apache.kafka:kafka-clients:3.9.0
//DEPS io.confluent:kafka-protobuf-serializer:7.7.1
//DEPS io.confluent:kafka-avro-serializer:7.7.1
//DEPS io.confluent:kafka-json-schema-serializer:7.7.1
//DEPS io.confluent:kafka-protobuf-serializer:7.7.1
//DEPS org.slf4j:slf4j-nop:2.0.16
//DEPS tech.allegro.schema.json2avro:converter:0.2.15
//DEPS com.google.protobuf:protobuf-java:3.25.4
//DEPS info.picocli:picocli:4.7.6


//FILES avro/key-schema.json=avro/key-schema.json
//FILES avro/value-schema.json=avro/value-schema.json
//FILES json-schema/value-schema.json=json-schema/value-schema.json
//FILES json-schema/key-schema.json=json-schema/key-schema.json
//FILES protobuf/key-schema.proto=protobuf/key-schema.proto
//FILES protobuf/value-schema.proto=protobuf/value-schema.proto


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
import java.util.concurrent.Callable;
import java.util.stream.Collectors;

import io.confluent.kafka.schemaregistry.protobuf.ProtobufSchema;
import picocli.CommandLine;
import picocli.CommandLine.Command;
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

    @Override
    public Integer call() throws Exception {
        Properties props = this.kafkaProperties();

        var url = System.getenv().getOrDefault("YOZEFU_API_URL", "https://api-adresse.data.gouv.fr/search/?q=%s");
        System.err.printf(" 🔩 The API is '%s'\n", url);
        var data = get(url, query);

        System.err.printf(" 📣 About to producing records to topic '%s', serialization type is '%s'\n", topic, type);
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


interface Into<K, V> {
    ProducerRecord<K, V> into(final String value, final String topic) throws Exception;

    default String generateKey() {
        return UUID.randomUUID().toString();
    }

    default String readResource(String path) throws Exception {
        try(var in = Into.class.getResourceAsStream(path)) {
            return new String(in.readAllBytes(), StandardCharsets.UTF_8);
        }
    }
}

class IntoText implements Into<String, String> {
    public ProducerRecord<String, String> into(final String value, final String topic) throws JsonProcessingException {
        var objectMapper = new ObjectMapper();
        var object = objectMapper.readTree(value);
        return new ProducerRecord<>(topic, this.generateKey(), object.get("properties").get("label").asText());
    }
}

class IntoJson implements Into<String, String> {
    public ProducerRecord<String, String> into(final String value, final String topic) {
        return new ProducerRecord<>(topic, generateKey(), value);
    }
}

class IntoJsonSchema implements Into<JsonNode, JsonNode> {
    public ProducerRecord<JsonNode, JsonNode> into(final String input, final String topic) throws Exception {
        var objectMapper = new ObjectMapper();
        var keySchemaString = readResource("/json-schema/key-schema.json");
        var valueSchemaString = readResource("/json-schema/value-schema.json");
        var keySchema = objectMapper.readTree(keySchemaString);
        var valueSchema = objectMapper.readTree(valueSchemaString);

        var key = TextNode.valueOf(generateKey());
        var keyEnvelope = JsonSchemaUtils.envelope(keySchema, key);

        var value = objectMapper.readTree(input);
        var valueEnvelope = JsonSchemaUtils.envelope(valueSchema, value);

        return new ProducerRecord<>(topic, keyEnvelope, valueEnvelope);
    }
}

class IntoAvro implements Into<GenericRecord, GenericRecord> {
    public ProducerRecord<GenericRecord, GenericRecord> into(final String input, final String topic) throws Exception {
        var keySchemaString = readResource("/avro/key-schema.json");
        var valueSchemaString = readResource("/avro/value-schema.json");

        Schema.Parser schemaParser = new Schema.Parser();
        Schema keySchema = schemaParser.parse(keySchemaString);
        Schema valueSchema = schemaParser.parse(valueSchemaString);
        JsonAvroConverter converter = new JsonAvroConverter();

        var keyString = String.format("{ \"id\": \"%s\", \"sunny\": %s }", generateKey(), new Random().nextBoolean());
        GenericData.Record key = converter.convertToGenericDataRecord(keyString.getBytes(), keySchema);
        GenericData.Record value = converter.convertToGenericDataRecord(input.getBytes(), valueSchema);
        return new ProducerRecord<>(topic, key, value);
    }
}

// TODO work in progress
class IntoProtobuf implements Into<Object, Object> {
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

class IntoMalformed implements Into<byte[], byte[]> {
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

class IntoInvalidJson implements Into<JsonNode, JsonNode> {
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

class IntoXml implements Into<String, String> {
    public ProducerRecord<String, String> into(final String input, final String topic) throws Exception {
        var objectMapper = new ObjectMapper();
        var xmlMapper = new XmlMapper();
        var value = objectMapper.readTree(input);
        return new ProducerRecord<>(topic, generateKey(), xmlMapper.writeValueAsString(value));
    }
}