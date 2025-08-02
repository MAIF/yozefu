# yozefu-command

[![Build](https://github.com/MAIF/yozefu/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/MAIF/yozefu/actions/workflows/build.yml)
[![The crate](https://img.shields.io/crates/v/yozefu-command.svg)](https://crates.io/crates/yozefu-command)


This library contains the `clap` commands of [Yozefu](https://github.com/MAIF/yozefu):
 - `configure` to access to the configuration file.
 - `create-filter` to crate a new search filter
 - `import-filter` to import the search filter to the tool.


The crate also exports a `headless` mode. It is the same application but without the usage of Ratatui. Results are printed to `stdout`.


## Usage

```bash
cargo add yozefu-command
```