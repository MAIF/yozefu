package serializers;
//JAVA 25+
//REPOS central,confluent=https://packages.confluent.io/maven
//DEPS org.apache.kafka:kafka-clients:4.1.0
//DEPS org.slf4j:slf4j-nop:2.0.16


import java.util.*;
import java.nio.charset.StandardCharsets;
import org.apache.kafka.clients.producer.*;
import io.confluent.kafka.schemaregistry.client.SchemaRegistryClient;
import io.confluent.kafka.schemaregistry.client.rest.entities.SchemaReference;

public interface Into<K, V> {
    ProducerRecord<K, V> into(final String value, final String topic) throws Exception;

    default String generateKey() {
        return UUID.randomUUID().toString();
    }

    default void registerSchemas(SchemaRegistryClient schemaRegistryClient) throws Exception {
    }

    default String readResource(String path) throws Exception {
        try(var in = Into.class.getResourceAsStream(path)) {
            return new String(in.readAllBytes(), StandardCharsets.UTF_8);
        }
    }
}