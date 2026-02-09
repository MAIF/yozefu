
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

//FILES avro/key-schema.json=../avro/key-schema.json
//FILES avro/value-schema.json=../avro/value-schema.json
//FILES avro/point-schema.json=../avro/point-schema.json

//SOURCES Into.java

package serializers;
import java.util.*;
import org.apache.kafka.clients.producer.*;
import org.apache.avro.generic.GenericData;
import org.apache.avro.Schema;
import org.apache.avro.generic.GenericRecord;
import tech.allegro.schema.json2avro.converter.JsonAvroConverter;
import io.confluent.kafka.schemaregistry.client.SchemaRegistryClient;
import io.confluent.kafka.schemaregistry.client.rest.entities.SchemaReference;
import io.confluent.kafka.schemaregistry.avro.AvroSchema;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.ObjectMapper;


public class IntoAvro implements Into<GenericRecord, GenericRecord> {
    
    
    public void registerSchemas(SchemaRegistryClient schemaRegistryClient) throws Exception {
        var keySchemaString = readResource("/avro/key-schema.json");
        var valueSchemaString = readResource("/avro/value-schema.json");
        var pointSchemaString = readResource("/avro/point-schema.json");
        var schemaParser = new Schema.Parser();

        schemaRegistryClient.register("public-french-addresses-key", new AvroSchema(keySchemaString));

        int pointSchemaId = schemaRegistryClient.register(
            "io.maif.yozefu.Point", 
            new AvroSchema(pointSchemaString)
        );

        var pointRef = new SchemaReference(
            "io.maif.yozefu.Point",
            "io.maif.yozefu.Point",
            1
        );

        var parsedValue = schemaRegistryClient.parseSchema(
          "AVRO",
          valueSchemaString,
          java.util.List.of(pointRef)
          ).orElseThrow(() -> new IllegalStateException("Failed to parse value schema"));

        schemaRegistryClient.register(
            "public-french-addresses-value",
            parsedValue
        );
    }

    public ProducerRecord<GenericRecord, GenericRecord> into(final String input, final String topic) throws Exception {
        var mapper = new ObjectMapper();
        var deserialized = mapper.readTree(input);
        //if(System.currentTimeMillis() % 2 == 0) {
        ((ObjectNode) deserialized).put("timestamp", 4284823489238492839489234892389428942984892.0);
        //}
        var keySchemaString = readResource("/avro/key-schema.json");
        var valueSchemaString = readResource("/avro/value-schema.json");
        var pointSchemaString = readResource("/avro/point-schema.json");

        Schema.Parser schemaParser = new Schema.Parser();
        var pointSchema = schemaParser.parse(pointSchemaString);

        GenericRecord child = new GenericData.Record(pointSchema);
        Schema keySchema = schemaParser.parse(keySchemaString);
        Schema valueSchema = schemaParser.parse(valueSchemaString);

        JsonAvroConverter converter = new JsonAvroConverter();
        GenericData.Record point = converter.convertToGenericDataRecord("""
            { "type": "Point", "coordinates": [40.23, 12.3] }""".getBytes(), pointSchema);

        var keyString = String.format("{ \"id\": \"%s\", \"sunny\": %s }", generateKey(), new Random().nextBoolean());
        GenericData.Record key = converter.convertToGenericDataRecord(keyString.getBytes(), keySchema);
        GenericData.Record value = converter.convertToGenericDataRecord(mapper.writeValueAsString(deserialized).getBytes(), valueSchema);
        value.put("geometry", point);
        return new ProducerRecord<>(topic, key, value);
    }
}
