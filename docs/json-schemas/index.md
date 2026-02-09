---
description: JSON schemas for important data structures used by Yōzefu.
---

# JSON schemas

these json schemas are for documentation purpose and give an overview of the data structures used by Yozefu.

| Name                  | Rust definition                                                                                                    | Schema definition                                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| Kafka record          | [kafka_record.rs](https://github.com/MAIF/yozefu/blob/main/crates/lib/src/kafka/kafka_record.rs)                   | [kafka-record.json](https://github.com/MAIF/yozefu/blob/main/docs/json-schemas/kafka-record.json)                   |
| Exported kafka record | [exported_kafka_record.rs](https://github.com/MAIF/yozefu/blob/main/crates/lib/src/kafka/exported_kafka_record.rs) | [exported-kafka-record.json](https://github.com/MAIF/yozefu/blob/main/docs/json-schemas/exported-kafka-record.json) |
| Input filter          | [mod.rs](https://github.com/MAIF/yozefu/blob/main/crates/lib/src/search/mod.rs)                                    | [filter-input.json](https://github.com/MAIF/yozefu/blob/main/docs/json-schemas/filter-input.json)                   |
| Result filter         | [mod.rs](https://github.com/MAIF/yozefu/blob/main/crates//lib/src/search/mod.rs)                                   | [filter-result.json](https://github.com/MAIF/yozefu/blob/main/docs/json-schemas/filter-result.json)                 |
| Configuration         | [global_config.rs](https://github.com/MAIF/yozefu/blob/main/crates/app/src/configuration/global_config.rs)         | [global-config.json](https://github.com/MAIF/yozefu/blob/main/docs/json-schemas/global-config.json)                 |
