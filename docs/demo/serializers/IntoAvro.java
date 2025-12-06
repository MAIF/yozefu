
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


public class IntoAvro implements Into<GenericRecord, GenericRecord> {
    public ProducerRecord<GenericRecord, GenericRecord> into(final String input, final String topic) throws Exception {
        var keySchemaString = readResource("/avro/key-schema.json");
        var valueSchemaString = readResource("/avro/value-schema.json");
        var pointSchemaString = readResource("/avro/point-schema.json");

        Schema.Parser schemaParser = new Schema.Parser();
        schemaParser.parse(pointSchemaString);
        Schema keySchema = schemaParser.parse(keySchemaString);
        Schema valueSchema = schemaParser.parse(valueSchemaString);

        JsonAvroConverter converter = new JsonAvroConverter();

        var keyString = String.format("{ \"id\": \"%s\", \"sunny\": %s }", generateKey(), new Random().nextBoolean());
        GenericData.Record key = converter.convertToGenericDataRecord(keyString.getBytes(), keySchema);
        GenericData.Record value = converter.convertToGenericDataRecord(input.getBytes(), valueSchema);
        return new ProducerRecord<>(topic, key, value);
    }
}
