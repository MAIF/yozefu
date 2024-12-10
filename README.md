# Yōzefu

<a href="https://github.com/MAIF/yozefu/releases"><img src="https://img.shields.io/github/v/release/MAIF/yozefu?style=flatd&color=f8be75&logo=GitHub"></a>
<a href="https://crates.io/crates/yozefu/"><img src="https://img.shields.io/crates/v/yozefu?logo=Rust"></a>
<a href="https://github.com/MAIF/yozefu/actions/workflows/build.yml"><img src="https://github.com/MAIF/yozefu/actions/workflows/build.yml/badge.svg" alt="Build status"/></a>
<a href="https://docs.rs/yozefu/"><img src="https://img.shields.io/docsrs/yozefu?logo=Rust"></a>
<a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/MSRV-1.80.1+-lightgray.svg?logo=rust" alt="Minimum supported Rust version: 1.80.1 or plus"/></a>
<a href="https://github.com/MAIF/yozefu/blob/main/LICENSE"><img src="https://img.shields.io/github/license/MAIF/yozefu" alt="Licence"/></a>


Yōzefu is an interactive terminal user interface (TUI) application for exploring data of a kafka cluster.
It is an alternative tool to [AKHQ](https://akhq.io/), [redpanda console](https://www.redpanda.com/redpanda-console-kafka-ui) or [the kafka plugin for JetBrains IDEs](https://plugins.jetbrains.com/plugin/21704-kafka).

The tool offers the following features:
 - A real-time access to data published to topics.
 - A search query language inspired from SQL providing a fine-grained way filtering capabilities.
 - Ability to search kafka records across multiple topics.
 - Support for extending the search engine with [user-defined filters](./docs/search-filter/README.md) written in WebAssembly ([Extism](https://extism.org/)).
 - The tool can be used as a terminal user interface or a CLI with the `--headless` flag.
 - One keystroke to export kafka records for further analysis.
 - Support for registering multiple kafka clusters, each with specific kafka consumer properties.


By default, [the kafka consumer is configured](https://github.com/MAIF/yozefu/blob/main/crates/command/src/command/main_command.rs#L318-L325) with the property `enable.auto.commit` set to `false`, meaning no kafka consumer offset will be published to kafka.


<a href="https://mcdostone.github.io/yozefu.mp4" target="_blank">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs/screenshots/dark.png">
    <img alt="Link to a demo video" src="./docs/screenshots/light.png">
  </picture>
</a>

## Limitations

 - The tool is designed only to consume kafka records. There is no feature to produce records or manage a cluster.
 - Serialization formats such as `json`, `xml` or plain text are supported. [Avro](https://avro.apache.org/) support is [experimental for now](./docs/schema-registry/README.md). [Protobuf](https://protobuf.dev/) is not supported.
 - The tool uses a ring buffer to store the [last 500 kafka records](./crates/tui/src/records_buffer.rs#L20).
 - There is probably room for improvement regarding the throughput (lot of `clone()` and deserialization).
 - Yozefu has been tested on MacOS Silicon but not on Windows or Linux. Feedback or contributions are welcome.


## Getting started

> [!NOTE]
> For a better visual experience, I invite you to install [Powerline fonts](https://github.com/powerline/fonts).

```bash
cargo install yozefu

# By default, it starts the TUI. 
# The default registered cluster is localhost
yozf --cluster localhost

# You can also start the tool in headless mode.
# It prints the key of each kafka record matching the query in real time
yozf --cluster localhost               \
    --headless                         \
    --topics "public-french-addresses" \
    --format "json"                    \
    'from begin value.properties.type contains "street" and offset < 356_234 limit 10' \
  | jq '.key'


# Use the `configure` command to define new clusters
yozf configure

# You can create search filters
yozf create-filter --language rust key-ends-with

# And import them
yozf import-filter path/to/key-ends-with.wasm
```

You can also download pre-build binaries from the [releases section](https://github.com/MAIF/yozefu/releases). [Attestions](https://github.com/MAIF/yozefu/attestations) are available:
```bash
gh attestation verify --repo MAIF/yozefu <file-path of downloaded artifact> 
```


## Try it

> [!NOTE]
> Docker is required to start a single node Kafka cluster on your machine. [JBang](https://www.jbang.dev/) is not required but recommended if you want to produce records with the schema registry.


```bash
# It clones this repository, starts a docker kafka node and produce some json records
curl -L "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/try-it.sh" | bash

yozf -c localhost
```


## Documentation

 - [The query language.](./docs/query-language/README.md)
 - [Creating a search filter.](./docs/search-filter/README.md)
 - [Configuring the tool with TLS.](./docs/tls/README.md)
 - [URL templates to switch to web applications.](./docs/url-templates/README.md)
 - [Schema registry.](./docs/schema-registry/README.md)
 - [Themes.](./docs/themes/README.md)
 - [Releasing a new version.](./docs/release/README.md)
 
