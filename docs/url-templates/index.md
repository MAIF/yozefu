---
description: URL templates to open Kafka records in web applications from the TUI.
---

# URL templates

In certain situations, you may need to view a Kafka record in a web browser. Yōzefu allows you to do so: select the Kafka record and press the <kbd>o</kbd> key (for **o**pen). This will open the corresponding URL in a new browser tab.

The tool uses a URL template from the configuration file. this template is defined in the `.clusters.<name-of-your-cluster>.url_template` property, where `<name-of-your-cluster>` is the specific cluster name you're using.

This list gives the different URL templates depending on the web application you use:

| Nom                                                                                    | URL template                                                                                                                                                                      |
|----------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [Confluent](https://confluent.cloud)                                                   | <https://confluent.cloud/environments/acme-environment/clusters/acme-cluster/topics/{topic}/message-viewer>                                                                         |
| [Control center](https://docs.confluent.io/platform/current/control-center/index.html) | <https://control-center.acme/clusters/acme-cluster/management/topics/{topic}/message-viewer>                                                                                        |
| [Redpanda Cloud](https://cloud.redpanda.com/)                                          | <https://cloud.redpanda.com/clusters/acme-cluster/topics/{topic}?p={partition}&s=1&o={offset}#messages>                                                                             |
| [Redpanda Console](https://www.redpanda.com/redpanda-console-kafka-ui)                 | <https://redpanda-console.acme/topics/{topic}?p={partition}&s=1&o={offset}#messages>                                                                                                |
| [AKHQ](https://akhq.io/)                                                               | <https://akhq.acme/cluster/{topic}/data?single=true&partition={partition}&offset={offset}{offset}>                                                                                  |
| [Kafka UI](https://docs.kafka-ui.provectus.io/)                                        | <https://kafka-ui.acme/ui/clusters/kafk/all-topics/{topic}/messages?limit=1&seekType=OFFSET&seekTo=0%3A%3A{offset}>                                                                 |
| [Kafdrop](https://github.com/obsidiandynamics/kafdrop)                                 | <https://kadrop.acme/topic/{topic}/messages?partition={partition}&offset={offset}&count=1>                                                                                          |
| [Kpow](https://factorhouse.io/kpow)                                                    | <https://kpow.acme/#/tenant/__kpow_global/cluster/acme-cluster/data/inspect?offset={offset}&topic={topic}&partition={partition}>                                                    |
| [Kouncil](https://kouncil.io/)                                                         | <https://kouncil.acme/topics/messages/{topic}>                                                                                                                                      |
| [Kafbat UI](https://ui.docs.kafbat.io/)                                                | <https://kafbat-ui.acme/ui/clusters/acme-cluster/all-topics/{topic}/messages?limit=1&mode=FROM_OFFSET&offset={offset}&partitions={partition}>                                       |
| [Aiven](https://aiven.io/kafka)                                                        | <https://console.aiven.io/account/{todo-account-id}/project/{todo-project-id}/services/{todo-service-id}/topics/{topic}/messages?format=json&offset={offset}&partition={partition}> |

At this time, [3 variables can be used](https://github.com/MAIF/yozefu/blob/main/crates/tui/src/component/ui.rs#L312-L318) in the URL template: `{topic}`, `{partition}` and `{offset}`.
