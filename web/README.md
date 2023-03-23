# Ambient Web

This crate provides a workspace for the web client for Ambient.

The client is compiled using the `wasm32-unknown-unknown` toolchain into a `.wasm` file, which is then imported as an ECMAScript module.

## Build Prerequisites

  - [wasm-pack](https://rustwasm.github.io/wasm-pack/)
  - [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
  - Node `>= v.19`
  - WebGPU supported web browser (Chrome Canary)

## Setup
```sh
cd www

# Installs the dependencies and the webpack dev server
npm install -d

cd ..
rustup target add wasm32-unknown-unknown
```

## Building

From the root run:

```
wasm-pack build client
```

## Running
```
cd www

npm run start
```

This will launch `webpack dev server` on port `8080`

Open chrome (or another browser which supports webgpu)

[localhost:8080](http://localhost:8080)

## Known Issues

- Bad CPU type in executable:

  Occurs on Mac M1 and M2 as the `wasm-pack` installer attempts to download `wasm-bindgen`.

  This is fixed by doing `cargo install -f wasm-bindgen` manually
