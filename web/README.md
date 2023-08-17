# Ambient Web

This crate provides a workspace for the web client for Ambient.

The client is compiled using the `wasm32-unknown-unknown` toolchain into a `.wasm` file, which is then imported as an ECMAScript module.

Note: you need to serve an ember using `ambient` for the web client to connect to.

## Build Prerequisites

- Node `>= v.19`
- WebGPU supported web browser (recent enough)

## Building and Serving

```sh
cargo campfire web serve
```

This build the client and launch `vite dev server` binding to `:5173`. See the command output for exact url.

Whenever a file changes the client will automatically rebuild and the changes will be reflected in the browser.

**Note**: the self-signed certificate is only valid for `127.0.0.1`

## Connecting

**Note**: Skip this section if you are connecting to a hosted Ember.

If using self-signed certificates, you need to toll Chrome to trust it.

```sh
cargo run -p campfire --features openssl -- web open-browser
```

**Note**: If you are on mac **make sure you close any existing Chrome instances using `Quit`**

<!-- After opening the client it will attempt connect to a locally running `ambient server` on `127.0.0.1:9000` (the default) -->

## Known Issues

- Bad CPU type in executable:

  Occurs on Mac M1 and M2 as the `wasm-pack` installer attempts to download `wasm-bindgen`.

  This is fixed by doing `cargo install -f wasm-bindgen` manually

- openssl issue :

  try: `export NODE_OPTIONS=--openssl-legacy-provider` on unix or `set NODE_OPTIONS=--openssl-legacy-provider` on windows
