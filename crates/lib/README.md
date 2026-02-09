# yozefu-lib

[![Build](https://github.com/MAIF/yozefu/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/MAIF/yozefu/actions/workflows/build.yml)
[![Yozefu Lib image](https://img.shields.io/crates/v/yozefu-lib.svg)](https://crates.io/crates/yozefu-lib)

This crate provides the core definitions for [Yozefu](https://github.com/MAIF/yozefu).

- [`KafkaRecord`](./src/kafka/kafka_record.rs), the main structure used everywhere representing a kafka record.
- Definitions of errors that can occur.
- Structures and functions to parse and execute search queries.

## Usage

```bash
cargo add yozefu-lib
```
