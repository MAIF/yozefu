---
description: Configuration options CLI flags and environment variables for Yōzefu
---

# Configuration

Yōzefu uses a workspace directory to store its configuration file, logs, and other data. By default, this directory is located at `yozf config get dir`. You can change the workspace directory using the `--config-dir` command-line option or the `YOZEFU_CONFIG_DIR` environment variable.



|               | Default                           |      CLI option |        Env variable |  Configuration file |
| ------------- | --------------------------------- | --------------: | ------------------: | ------------------: |
| Workspace     | `~/.config/io.maif.yozefu/`       |  `--config-dir` | `YOZEFU_CONFIG_DIR` |                  No |
| Configuration | `${workspace}/config.json`        | `--config-file` |                 N/A |                  No |
| Logs          | `${workspace}/application.log`    |    `--log-file` |   `YOZEFU_LOG_FILE` |         `/log_file` |
| Export folder | `$PWD/export-{datetime-now}.json` |      `--output` |                  No | `/export_directory` |




## `config.json` file


Yōzefu uses a JSON configuration file to set various options. By default, it looks for a file named `config.json` in the workspace directory.


| Key                                                                                      | Type                  | Examples                                                |
| ---------------------------------------------------------------------------------------- | --------------------- | ------------------------------------------------------- |
| `default_url_template` <br/> Placeholder URL to open a Kafka record in the browser.      | String                | `http://localhost/cluster/{topic}/{partition}/{offset}` |
| `initial_query`        <br/> Initial search query when starting the UI.                  | String                | `from end - 10`                                         |
| `theme`                <br/> TUI theme.                                                  | String                | `light`                                                 |
| `highlighter_theme`    <br/> Theme for syntax highlighting.                              | String                | `InspiredGitHub`                                        |
| `clusters`             <br/> Kafka properties per  <a href="#kafka-cluster">cluster</a>. | Object                | —                                                       |
| `consumer`             <br/> Default configuration for the Kafka consumer.               | Object                | `{ buffer_capacity: 1000, timeout_in_ms: 10 }`          |
| `default_kafka_config` <br/> Default Kafka properties inherited by every cluster.        | Map\<String, String\> | `{"fetch.min.bytes": "10000"}`                          |
| `history`              <br/> Past search queries.                                        | Array\<string\>       | `["from end - 10"]`                                     |
| `show_shortcuts`       <br/> Whether to show shortcuts.                                  | Boolean               | `true`                                                  |
| `export_directory`     <br/> Directory for exports.                                      | String                | `./yozefu-exports`                                      |
| `log_file`             <br/> File path to write logs.                                    | String                | `/path/to/log/file.log`                                 |
| `timestamp_format`     <br/> Display timestamps as date-time or relative.                | `DateTime` or `Ago`   | `DateTime`                                              |


## Kafka cluster


| Key                                                                             | Type                  | Examples                                          |
| ------------------------------------------------------------------------------- | --------------------- | ------------------------------------------------- |
| `url_template`    <br/>  Placeholder URL to open a Kafka record in the browser. | String                | `http://dev/cluster/{topic}/{partition}/{offset}` |
| `schema_registry` <br/> Schema registry settings for this cluster.              | Object                | `{}`                                              |
| `kafka`           <br/> Kafka consumer properties for this cluster              | Map\<String, String\> | `{}`                                              |
| `consumer`        <br/> configuration for the Yozefu consumer.                  | Object                | `{ buffer_capacity: 10, timeout_in_ms: 1 }`        |


For more details, see the [configuration json schema](../json-schemas/).