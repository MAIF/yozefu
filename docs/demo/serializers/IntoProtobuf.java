package serializers;
//JAVA 25+
//REPOS central,confluent=https://packages.confluent.io/maven
//DEPS com.fasterxml.jackson.core:jackson-databind:2.20.0
//DEPS com.fasterxml.jackson.dataformat:jackson-dataformat-xml:2.20.0
//DEPS org.apache.kafka:kafka-clients:4.0.0
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
import io.confluent.kafka.schemaregistry.client.SchemaRegistryClient;
import io.confluent.kafka.schemaregistry.client.rest.entities.SchemaReference;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.util.*;

public class IntoProtobuf implements Into<DynamicMessage, DynamicMessage> {
    
    private ProtobufSchema keySchema;
    private ProtobufSchema valueSchema;
    private String topic;
    
    public IntoProtobuf() throws Exception {
        var keySchemaString = readResource("/protobuf/key-schema.proto");
        var valueSchemaString = readResource("/protobuf/value-schema.proto");
        this.keySchema = new ProtobufSchema(keySchemaString);
        this.valueSchema = new ProtobufSchema(valueSchemaString);
    }
    
    public void setTopic(String topic) {
        this.topic = topic;
    }
    
    @Override
    public void registerSchemas(SchemaRegistryClient schemaRegistryClient) throws Exception {
        var topicKey = topic + "-key";
        var topicValue = topic + "-value";
        try {
            schemaRegistryClient.register(topicKey, keySchema);
        } catch (Exception e) {
            System.err.printf(" Key schema might already exist for %s: %s\n", topicKey, e.getMessage());
        }
        try {
            schemaRegistryClient.register(topicValue, valueSchema);
        } catch (Exception e) {
            System.err.printf(" Value schema might already exist for %s: %s\n", topicValue, e.getMessage());
        }
    }
    
    public ProducerRecord<DynamicMessage, DynamicMessage> into(final String input, final String topic) throws Exception {
        this.topic = topic;
        ObjectMapper mapper = new ObjectMapper();
        JsonNode feature = mapper.readTree(input);
        
        var keyString = String.format("{\"id\": \"%s\"}", this.generateKey());
        DynamicMessage key = (DynamicMessage) ProtobufSchemaUtils.toObject(keyString, keySchema);
        
        var protobufValue = convertFeatureToProtobufValue(feature);
        DynamicMessage value = (DynamicMessage) ProtobufSchemaUtils.toObject(protobufValue, valueSchema);
        
        return new ProducerRecord<>(topic, key, value);
    }
    
    private String convertFeatureToProtobufValue(JsonNode feature) throws Exception {
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode protobufJson = mapper.createObjectNode();
        
        protobufJson.put("type", feature.get("type").asText());
        
        JsonNode geometry = feature.get("geometry");
        if (geometry != null) {
            ObjectNode protobufGeometry = mapper.createObjectNode();
            protobufGeometry.put("type", geometry.get("type").asText());
            
            JsonNode coordinates = geometry.get("coordinates");
            if (coordinates != null && coordinates.isArray()) {
                var coordsArray = mapper.createArrayNode();
                for (JsonNode coord : coordinates) {
                    coordsArray.add(coord.asDouble());
                }
                protobufGeometry.putArray("coordinates").addAll(coordsArray);
            }
            protobufJson.putObject("geometry").setAll(protobufGeometry);
        }
        
        JsonNode properties = feature.get("properties");
        if (properties != null) {
            ObjectNode protobufProperties = mapper.createObjectNode();
            
            if (properties.has("label")) protobufProperties.put("label", properties.get("label").asText());
            if (properties.has("score")) protobufProperties.put("score", properties.get("score").asDouble());
            if (properties.has("id")) protobufProperties.put("id", properties.get("id").asText());
            if (properties.has("name")) protobufProperties.put("name", properties.get("name").asText());
            if (properties.has("postcode")) protobufProperties.put("postcode", properties.get("postcode").asText());
            if (properties.has("citycode")) protobufProperties.put("citycode", properties.get("citycode").asText());
            if (properties.has("x")) protobufProperties.put("x", properties.get("x").asDouble());
            if (properties.has("y")) protobufProperties.put("y", properties.get("y").asDouble());
            if (properties.has("city")) protobufProperties.put("city", properties.get("city").asText());
            if (properties.has("context")) protobufProperties.put("context", properties.get("context").asText());
            if (properties.has("type")) protobufProperties.put("type", properties.get("type").asText());
            if (properties.has("importance")) protobufProperties.put("importance", properties.get("importance").asDouble());
            if (properties.has("street")) protobufProperties.put("street", properties.get("street").asText());
            if (properties.has("banId")) protobufProperties.put("banId", properties.get("banId").asText());
            if (properties.has("population")) protobufProperties.put("population", properties.get("population").asDouble());
            if (properties.has("municipality")) protobufProperties.put("municipality", properties.get("municipality").asText());
            if (properties.has("locality")) protobufProperties.put("locality", properties.get("locality").asText());
            if (properties.has("oldcitycode")) protobufProperties.put("oldcitycode", properties.get("oldcitycode").asText());
            if (properties.has("oldcity")) protobufProperties.put("oldcity", properties.get("oldcity").asText());
            
            protobufJson.putObject("properties").setAll(protobufProperties);
        }
        
        return mapper.writeValueAsString(protobufJson);
    }
}
