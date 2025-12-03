<p align="center" style="width: 150px">
<picture>
    <source width="150" media="(prefers-color-scheme: dark)" srcset="https://github.com/MAIF/yozefu/raw/main/docs/public/logo.svg">
    <img width="150" alt=" Pixel art portrait of a serious-faced person in grayscale above the text 'Yozefu'" src="https://github.com/MAIF/yozefu/raw/main/docs/public/logo.svg">
  </picture>
</p>

<p align="center">
<a href="https://crates.io/crates/yozefu/"><img alt="Yozefu crate.io page" src="https://img.shields.io/crates/v/yozefu?logo=Rust"></a>
<a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/MSRV-1.85.0+-lightgray.svg?logo=rust" alt="Minimum supported Rust version: 1.85.0 or plus"/></a>
<a href="https://github.com/MAIF/yozefu/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="Licence"/></a>
<a href="https://ratatui.rs/"><img src="https://ratatui.rs/built-with-ratatui/badge.svg" alt="Built With Ratatui"/></a>


</p>

Yōzefu is an interactive terminal user interface (TUI) application for exploring data of a kafka cluster.
It is an alternative tool to [AKHQ](https://akhq.io/), [redpanda console](https://www.redpanda.com/redpanda-console-kafka-ui) or [the kafka plugin for JetBrains IDEs](https://plugins.jetbrains.com/plugin/21704-kafka).

The tool offers the following features:
 - A real-time access to data published to topics.
 - A [search query language](https://github.com/MAIF/yozefu/tree/main/docs/query-language/README.md) inspired from SQL providing a fine-grained way filtering capabilities.
 - Ability to search kafka records across multiple topics.
 - Support for extending the search engine with [user-defined filters](https://github.com/MAIF/yozefu/tree/main/docs/search-filter/README.md) written in WebAssembly ([Extism](https://extism.org/)).
 - The tool can be used as a terminal user interface or a CLI with the `--headless` flag.
 - One keystroke to export kafka records for further analysis.
 - Support for registering multiple kafka clusters, each with specific kafka consumer properties.


By default, [the kafka consumer is configured](https://github.com/MAIF/yozefu/blob/main/crates/command/src/command/main_command.rs#L318-L325) with the property `enable.auto.commit` set to `false`, meaning no kafka consumer offset will be published to kafka.



<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://vhs.charm.sh/vhs-UpIJD2h92vKkj01XSS0r0.gif">
  <img alt="Demo of Yozefu: The user selects a topic and sees and real time new records published to Kafka." src="https://vhs.charm.sh/vhs-1oh0ovd0DaUfvKLTx4iZTo.gif">
</picture>

## Limitations

 - The tool is designed only to consume kafka records. There is no feature to produce records or manage a cluster.
 - Serialization formats such as `json`, `xml` or plain text are supported. [Avro](https://avro.apache.org/) support is [experimental for now](https://github.com/MAIF/yozefu/tree/main/docs/schema-registry/README.md). [Protobuf](https://protobuf.dev/) is not supported.
 - The tool uses a ring buffer to store the [last 500 kafka records](https://github.com/MAIF/yozefu/blob/main/crates/tui/src/records_buffer.rs#L17).
 - There is probably room for improvement regarding the throughput (lot of `clone()` and deserialization).
 - Yozefu has been tested on macOS Silicon but not on Windows or Linux. Feedback or contributions are welcome.


## Getting started

<!--
> [!NOTE]
> For a better visual experience, I invite you to install [Powerline fonts](https://github.com/powerline/fonts).
> -->

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo install --locked yozefu

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
gh attestation verify --repo MAIF/yozefu $(which yozf)

brew install yozefu

yay -S yozefu

nix run github:MAIF/yozefu
```


## Try it

> [!NOTE]
> Docker is required to start a single node Kafka cluster on your machine. [JBang](https://www.jbang.dev/) is not required but recommended if you want to produce records with the schema registry.


```bash
# It clones this repository, starts a docker kafka node and produces json records
curl -L "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/try-it.sh" | bash

yozf -c localhost
```


## Documentation

To check out docs, visit [https://maif.github.io/yozefu/](https://maif.github.io/yozefu/).
 
## Screenshots

<table>
  <tr>
    <td>
      <img alt=" A TUI displays a list of topics on the left, a search bar below, and records with timestamps and values in a table on the right. " src="https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/public/topics.png">
    </td>
    <td>
      <img alt="A TUI listing the consumed kafka records with their timestamps, their partition, their offset and an excerpt of the key and the value." src="https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/public/records.png">
    </td>
  </tr>
  <tr>
    <td>
      <img alt="View of a selected kafka records" src="https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/public/record.png">
    </td>
    <td>
      <img alt="View of the help page" src="https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/public/help.png">
    </td>
  </tr>
</table>

