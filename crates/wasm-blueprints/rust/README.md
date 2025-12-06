# Rust search filter

<a href="https://maif.github.io/yozefu/search-filter/"><img src="https://img.shields.io/badge/Doc-Creating_a_search_filter-black.svg?logo=github" alt="Link explaining how to write a search filter"/></a>
<a href="https://github.com/extism/rust-pdk"><img src="https://img.shields.io/badge/Doc-Extism_rust_PDK-darkblue.svg" alt="Link to Extism Rust PDK"/></a>

Blueprint project to write a search filter in Rust.
To build the WebAssembly module:
```bash
make build
yozf import-filter module.wasm --name 'key-ends-with'
```