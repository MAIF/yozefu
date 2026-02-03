---
description: How to configure and use schema registries
---

# Schema registry

> [!WARNING]
> The support of the schema registry is **highly experimental**. Contributions and feedback are welcome.



| Types       | Support                   |
| ----------- | :------------------------ |
| Json schema | Experimental              |
| Avro        | Experimental              |
| Protobuf    | Looking for contributions |



You can configure the tool to use a schema registry. Open the configuration file and add a `schema_registry` entry to your cluster:

```shell
EDITOR=nano yozf configure
```
```json{4}
{
    "clusters": {
        "localhost": {
            "url_template": "http://localhost:9000/ui/kafka-localhost-server/topic/{topic}/data?single=true&partition={partition}&offset={offset}",
            "schema_registry": {
              "url": "http://localhost:8081"
            },
            "kafka": {
              "bootstrap.servers": "localhost:9092",
              "security.protocol": "plaintext",
              "broker.address.family": "v4"
            }
        }
    }
}
```





## Authentication

If the schema registry is protected by an authentication, you can specify the `Authorization` header for the schema registry client:

```json{4-6}
{
    "schema_registry": {
        "url": "https://acme-schema-registry:8081",
        "headers": {
            "Authorization": "Basic am9obkBleGFtcGxlLmNvbTphYmMxMjM="
        }
    }
}
```