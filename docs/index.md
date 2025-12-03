---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "Yōzefu,"
  text: "A TUI to explore Kafka clusters"
  image:
    src: /logo-without-name.svg
    alt: Yozefu

  actions:
    - theme: brand
      text: Getting started
      link: /getting-started/

features:
  - icon: 🔌
    title: Real-time + Multi-topic
    details: Live access to records and search across multiple topics.
    link: /what-is-yozefu
  - icon: 🔎
    title: Powerful Querying
    details: SQL-inspired query language with fine-grained filters, extensible via WebAssembly.
    link: /query-language/
  - icon: 🖥️
    title: Dual Modes
    details: Use as interactive TUI or CLI with the `--headless` flag.
    link: /what-is-yozefu/#a-tui-and-cli
  - icon: 📤
    title: One-key Export
    details: Quickly export Kafka records for deeper analysis.
    link: /keybindings/
---


![The user selects a topic and sees and real time new records published to Kafka.](https://vhs.charm.sh/vhs-UpIJD2h92vKkj01XSS0r0.gif){.dark-only .gif}
![The user selects a topic and sees and real time new records published to Kafka.](https://vhs.charm.sh/vhs-1oh0ovd0DaUfvKLTx4iZTo.gif){.light-only .gif}