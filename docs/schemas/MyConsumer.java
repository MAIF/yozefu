//JAVA 21+
//REPOS central,confluent=https://packages.confluent.io/maven
//DEPS org.apache.kafka:kafka-clients:3.9.0
//DEPS org.slf4j:slf4j-nop:2.0.16
//DEPS info.picocli:picocli:4.7.6

import org.apache.kafka.clients.consumer.ConsumerConfig;
import org.apache.kafka.clients.consumer.KafkaConsumer;

import java.util.*;

import java.io.FileInputStream;
import java.io.IOException;
import java.nio.file.Path;
import java.time.Duration;
import java.util.concurrent.Callable;

import picocli.CommandLine;
import picocli.CommandLine.Command;

@Command(name = "MyConsumer.java", version = "1.0.0", mixinStandardHelpOptions = true,
        description = "Tool to consume kafka records."
)
class MyConsumer implements Callable<Integer> {

    @CommandLine.Option(names = {"--properties"}, description = "Properties file for creating the kafka producer")
    private Optional<Path> properties = Optional.empty();

    @CommandLine.Parameters(description = "Name of the topic to consume", defaultValue = "public-french-addresses")
    private String topic;

    @Override
    public Integer call() throws Exception {
        Properties props = this.kafkaProperties();
        System.err.println(" 📣 About to consume records from topic '%s'");
        this.consume(new KafkaConsumer<byte[], byte[]>(props), this.topic);
        return 0;
    }

    public Properties kafkaProperties() {
        Properties props = new Properties();
        props.putIfAbsent(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");
        props.putIfAbsent(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.ByteArrayDeserializer");
        props.putIfAbsent(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, "org.apache.kafka.common.serialization.ByteArrayDeserializer");
        props.putIfAbsent(ConsumerConfig.GROUP_ID_CONFIG, "yozefu-my-consumer");
        props.putIfAbsent(ConsumerConfig.SESSION_TIMEOUT_MS_CONFIG, "6000");
        props.putIfAbsent(ConsumerConfig.SESSION_TIMEOUT_MS_CONFIG, "6000");
        props.putIfAbsent(ConsumerConfig.HEARTBEAT_INTERVAL_MS_CONFIG, "3000");
        props.putIfAbsent(ConsumerConfig.AUTO_COMMIT_INTERVAL_MS_CONFIG, "200");
        props.putIfAbsent(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, true);
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

    public static <K, V> void consume(final KafkaConsumer<K, V> consumer, final String topic) throws Exception {
        consumer.subscribe(Collections.singletonList(topic));
        while (true) {
            var records = consumer.poll(Duration.ofMillis(100));
            for (var record : records) {
                System.out.printf("topic = %s, offset = %d, key = %s, value = %s%n", topic, record.offset(), record.key(), record.value());
            }
        }
    }

    public static void main(String[] args) {
        int exitCode = new CommandLine(new MyConsumer())
                .setCaseInsensitiveEnumValuesAllowed(true)
                .execute(args);
        System.exit(exitCode);
    }

}
