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

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.util.Random;

public class Feeder {

    private final long seed;
    private final ObjectMapper mapper;

    public Feeder(long seed) {
        this.seed = seed;
        this.mapper = new ObjectMapper();
    }

    public ObjectNode generateJson(int index) {
        Random rnd = new Random(seed + index);
        double density = 0.2 + rnd.nextDouble() * 0.8;

        int schemaType = rnd.nextInt(4); // 0=user, 1=transaction, 2=product, 3=event
        return switch (schemaType) {
            case 0 -> generateUserJson(index, rnd, density);
            case 1 -> generateTransactionJson(index, rnd, density);
            case 2 -> generateProductJson(index, rnd, density);
            default -> generateEventJson(index, rnd, density);
        };
    }

    private ObjectNode generateUserJson(int id, Random rnd, double density) {
        ObjectNode json = mapper.createObjectNode();
        json.put("type", "user");
        json.put("id", id);
        if (rnd.nextDouble() < density) json.put("username", "user_" + rnd.nextInt(100000));
        if (rnd.nextDouble() < density) json.put("email", "user" + rnd.nextInt(1000) + "@example.com");
        if (rnd.nextDouble() < density) json.put("country", getRandomCountry(rnd));
        return json;
    }

    private ObjectNode generateTransactionJson(int id, Random rnd, double density) {
        ObjectNode json = mapper.createObjectNode();
        json.put("type", "transaction");
        json.put("id", id);
        if (rnd.nextDouble() < density) json.put("amount", rnd.nextDouble() * 10000);
        if (rnd.nextDouble() < density) json.put("currency", getRandomCurrency(rnd));
        if (rnd.nextDouble() < density) json.put("status", getRandomStatus(rnd));
        return json;
    }

    private ObjectNode generateProductJson(int id, Random rnd, double density) {
        ObjectNode json = mapper.createObjectNode();
        json.put("type", "product");
        json.put("id", id);
        if (rnd.nextDouble() < density) json.put("name", "Product-" + rnd.nextInt(9999));
        if (rnd.nextDouble() < density) json.put("price", rnd.nextDouble() * 500);
        if (rnd.nextDouble() < density) json.put("inStock", rnd.nextBoolean());
        return json;
    }

    private ObjectNode generateEventJson(int id, Random rnd, double density) {
        ObjectNode json = mapper.createObjectNode();
        json.put("type", "event");
        json.put("id", id);
        if (rnd.nextDouble() < density) json.put("eventType", getRandomEventType(rnd));
        if (rnd.nextDouble() < density) json.put("timestamp", System.currentTimeMillis() - rnd.nextInt(1_000_000));
        if (rnd.nextDouble() < density) json.put("userId", rnd.nextInt(10000));
        return json;
    }

    // Helpers

    private String getRandomCountry(Random rnd) {
        String[] countries = {"US", "DE", "FR", "IN", "CN", "BR", "NG", "JP"};
        return countries[rnd.nextInt(countries.length)];
    }

    private String getRandomCurrency(Random rnd) {
        String[] currencies = {"USD", "EUR", "JPY", "INR", "BRL", "GBP"};
        return currencies[rnd.nextInt(currencies.length)];
    }

    private String getRandomStatus(Random rnd) {
        String[] statuses = {"pending", "completed", "failed", "refunded"};
        return statuses[rnd.nextInt(statuses.length)];
    }

    private String getRandomEventType(Random rnd) {
        String[] events = {"click", "view", "purchase", "login", "logout"};
        return events[rnd.nextInt(events.length)];
    }

    public static void main(String[] args) {
        Feeder generator = new Feeder(98765L);
        for (int i = 0; i < 10; i++) {
            System.out.println(generator.generateJson(i).toPrettyString());
        }
    }
}
