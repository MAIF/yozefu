<div align="center">
  <img width="64px" src="https://upload.wikimedia.org/wikipedia/commons/1/1f/WebAssembly_Logo.svg" alt="logo of WebAssembly"/>
  <h1>Creating a search filter.</h1>
</div>

Refer to [the documentation](../../docs/search-filter/README.md) for more details. 2 blueprints are available to implement your search filter:
 - Rust
 - Golang

Your favorite programming language is not listed above? Feel free to contribute with another blueprint. Take a look at [Extism](https://extism.org/) to see if it supports your language.


## Creating a new blueprint

For a good develop experience, please respect the following rules:
 1. The blueprint must implement the `key-ends-with` search filter as an example.
 2. A `Makefile` must be present. Feel free to copy/paste one of [the existing ones](./rust/Makefile) and adapt it.
 3. Running `make build` must create a wasm file named `module.wasm`.
 4. A `README.md` must be present. You can also take inspiration from [the existing ones](./rust/README.md).
 5. The `Makefile` must include a `test` recipes running some basic tests. Feel free to copy/paste one of [the existing ones](./rust/Makefile) and adapt it.
