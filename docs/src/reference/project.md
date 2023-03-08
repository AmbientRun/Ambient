# Project

All Ambient projects must have an `ambient.toml` project manifest that describes their functionality. This format is in flux, but is inspired by Rust's `Cargo.toml`.

At present, dependencies are _not_ supported, but this will change in future.

## WebAssembly

All `.wasm` components in the `build/{client, server}` directory will be loaded for the given target, regardless of provenance. The `.wasm` filenames must be snake-case ASCII identifiers, like the `id` in the manifest.

This means any `.wasm` that implements the Ambient [WIT interface](https://github.com/AmbientRun/Ambient/tree/main/crates/wasm/wit) and targets WASI snapshot 2 (or uses an adapter that targets WASI snapshot 2) should run within Ambient.

As a convenience for Rust users, Ambient will automatically build a `Cargo.toml` at the root of your project, if present, as `wasm32-wasi` for both the `client` and `server` features. The resulting WASM bytecode files are then converted to components and placed in `build/{client, server}`.

The process it takes is equivalent to these commands:

```sh
cd your_project
cargo build --target wasm32-wasi --features client
wasm-tools component new target/wasm32-wasi/debug/your_project.wasm -o build/client/your_project.wasm --adapt wasi_snapshot_preview1.wasm
cargo build --target wasm32-wasi --features server
wasm-tools component new target/wasm32-wasi/debug/your_project.wasm -o build/server/your_project.wasm --adapt wasi_snapshot_preview1.wasm
```

using [wasm-tools](https://github.com/bytecodealliance/wasm-tools) and a bundled version of the [preview2-prototyping WASI adapter](https://github.com/bytecodealliance/preview2-prototyping).

## Reference

The full structure for `ambient.toml` is shown below with a sample:

<!-- TODO: autogenerate with generate-docs -->

```toml
{{#include ambient.sample.toml}}
```
