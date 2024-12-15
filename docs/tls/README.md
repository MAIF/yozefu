# TLS Support
<p>
<a href="https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md">
        <img src="https://img.shields.io/badge/librdkafka-Global_configuration_properties-black.svg?logo=github"></a> <a href="https://github.com/confluentinc/librdkafka/wiki/Using-SSL-with-librdkafka#configure-librdkafka-client">
        <img src="https://img.shields.io/badge/librdkafka-Configure_librdkafka_client-black.svg?logo=github"></a>
</p>



This page helps you configure TLS settings for different providers.
The steps are always the same:
1. Open the configuration with `yozf configure`
2. Edit the configuration file by adding a new cluster.
3. Save the file and run start the tool `yozf -c my-cluster`

If you use any of the following properties:`ssl.ca.location`, `ssl.certificate.location`, `ssl.key.location`, make sure to provide an absolute path, using `~` in the path doesn't work.

> [!WARNING]
> `SASL_SSL` security protocol is not available for `aarch64-unknown-linux-gnu` and `windows` targets. I'm facing some compilation issues.


## Confluent

To connect to a confluent kafka cluster:

1. Open https://confluent.cloud/environments
2. Select your cluster.
3. Click on **Clients** in the left menu.
4. Click on **Set up a new client**
5. Choose a Rust client.
6. Follow the instructions to generate an API key.
7. Open the configuration file: `yozf configure`
8. Edit the configuration:
```json
{
  "clusters": {
    "confluent": {
      "url_template": "https://confluent.cloud/environments/<environment>/clusters/<cluster>/topics/{topic}/message-viewer",
      "kafka": {
        "bootstrap.servers": "<server>.confluent.cloud:9092",
        "security.protocol": "SASL_SSL",
        "sasl.mechanisms": "PLAIN",
        "sasl.username": "<username>",
        "sasl.password": "<password",
        "session.timeout.ms": "45000"
      }
    }
  }
}
```

9. Save the configuration and start the tool:
```bash
yozf -c 'confluent' --headless --topics 'hello-world' 'from begin'
```


## Redpanda

1. Open https://cloud.redpanda.com/clusters
2. On the **Overview page**, select the **Kafka API** tab in the **How to connect** panel.
3. Generate new SASL credentials.
4. Edit the configuration:

```json
{
  "clusters": {
    "redpanda": {
      "url_template": "https://cloud.redpanda.com/clusters/<cluster>/topics/{topic}?p=-1&s=1&o={offset}#messages",
      "kafka": {
        "bootstrap.servers": "<cluster>.any.eu-central-1.mpx.prd.cloud.redpanda.com:9092",
        "security.protocol": "SASL_SSL",
        "sasl.mechanisms": "PLAIN",
        "sasl.username": "<username>",
        "sasl.mechanisms": "SCRAM-SHA-256",
        "sasl.password": "<password>"
      }
    }
  }
}
```





<!--

## AWS MSK

For more details, refer to the documentation: https://docs.aws.amazon.com/msk/latest/developerguide/what-is-msk.html

```json
{
  "clusters": {
    "acme": {
      "url_template": "http://akhq.acme/cluster/{topic}/data?single=true&partition={partition}&offset={offset}",
      "kafka": {
        "bootstrap.servers": "kafka-1.acme:9092,kafka-2.acme:9092",
        "security.protocol": "SASL_SSL",
        "sasl.mechanism": "AWS_MSK_IAM",
      }
    }
  }
}
```

-->


## Mutual TLS

For more details about Mutual TLS, refer to the documentation: https://docs.confluent.io/platform/current/kafka/configure-mds/mutual-tls-auth-rbac.html.Certificates.
Please note that, according to [the documentation](https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md), certificates must be in PEM format.
```json
{
  "clusters": {
    "acme": {
      "url_template": "http://akhq.acme/cluster/{topic}/data?single=true&partition={partition}&offset={offset}",
      "kafka": {
        "bootstrap.servers": "kafka-1.acme:9092,kafka-2.acme:9092",
        "security.protocol": "SSL",
        "ssl.ca.location": "/absolute-path/to/ca-certificate.pem",
        "ssl.certificate.location": "/absolute-path/to/certificate.pem",
        "ssl.key.location": "/absolute-path/to/client.key",
        "ssl.key.password": "<key-password>",
      }
    }
  }
}
```



## Cloud providers


[Contributions are welcomed](https://github.com/MAIF/yozefu/edit/main/docs/tls.md) to improve this page.


| Provider              | Tested  | Documentation                                                                                                                 |
| --------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Google Cloud Platform | `false` | https://cloud.google.com/managed-service-for-apache-kafka/docs/quickstart#cloud-shell                                         |
| Amazon Web Services   | `false` | https://docs.aws.amazon.com/msk/latest/developerguide/produce-consume.html                                                    |
| Microsoft Azure       | `false` | https://learn.microsoft.com/fr-fr/azure/event-hubs/azure-event-hubs-kafka-overview                                            |
| DigitalOcean          | `false` | https://docs.digitalocean.com/products/databases/kafka/how-to/connect/                                                        |
| OVH                   | `false` | https://help.ovhcloud.com/csm/en-ie-public-cloud-databases-kafka-getting-started?id=kb_article_view&sysparm_article=KB0048944 |