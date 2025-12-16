# What is Yōzefu?


Yōzefu is designed to be a fast and efficient tool to explore Kafka clusters from the terminal. It aims to provide a user-friendly interface while leveraging the power of the terminal for quick navigation and operations. More details on this [blog post](https://mcdostone.github.io/articles/why-yozefu/).


Yōzefu is a reference to the main character of **The Trial**, the novel of Franz Kafka.


## Features

Achraf offers the following features:
 - A real-time access to data published to topics.
 - A [search query language](/query-language/) inspired from SQL providing a fine-grained way filtering capabilities.
 - Ability to search kafka records across multiple topics.
 - Support for extending the search engine with [user-defined filters](/search-filter/) written in WebAssembly ([Extism](https://extism.org/)).
 - The tool can be used as a terminal user interface or a CLI with the `--headless` flag.
 - One keystroke to export kafka records for further analysis.
 - Support for registering multiple kafka clusters, each with specific kafka consumer properties.


## A TUI and a CLI

Yōzefu can be used as an interactive terminal user interface (TUI) or as a command-line interface in headless mode.

**TUI mode**
```shell
yozf --cluster preproduction \
-t my.topic.of.books \
"from begin where key == '978-2070315932' limit 1"
```

**Headless mode**
```shell
yozf --headless \
-c preproduction \
-t my.topic.of.books \
"from begin where key == '978-2070315932' limit 1" \
--format simple
```

## Limitations

 - The tool is designed only to consume kafka records. There is no feature to produce records or manage a cluster.
 - Serialization formats such as json, xml or plain text are supported. Avro support is experimental for now. Protobuf is not supported.
 - Yōzefu gives you the feeling that every kafka records stays in memory but in reality, it uses a ring buffer to store only the last 500 kafka records.
 - There is probably room for improvement regarding the throughput (lot of clone() and deserialization).
 - Yōzefu has been tested on macOS Silicon but not on Windows or Linux. Feedback or contributions are welcome.


## Screenshots

<img width="3455" height="2158" src="/records.avif" alt="Screenshot of the records view, a table displaying kafka records" />
<img width="3455" height="2161" src="/record.avif" alt="Screenshot of the record view, detailed view of a single kafka record" />
<img width="3455" height="2157" src="/topics.avif" alt="Screenshot of the topic view, a list of kafka topics you can consume" />
<img width="3455" height="2161" src="/help.avif" alt="Screenshot of the help view, displaying keybindings, the configuration and the syntax of the query language" />