# Ambient Web

This crate provides a workspace for the web client for Ambient.

The client is compiled using the `wasm32-unknown-unknown` toolchain into a `.wasm` file, which is then imported as an ECMAScript module.

## Build Prerequisites

- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- Node `>= v.19`
- WebGPU supported web browser (recent enough)

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
wasm-pack build client --dev
```

## Running

```
cd www

npm run dev
```

This will launch `vite dev server` and bind to `:5173`. See the command output for exact url

**Note**: the self-signed certificate is only valid for `127.0.0.1`

## Connecting

If using self-signed certificates, you need to toll Chrome to trust it

```sh
./scripts/launch_chrome.sh
```

## Known Issues

- Bad CPU type in executable:

  Occurs on Mac M1 and M2 as the `wasm-pack` installer attempts to download `wasm-bindgen`.

  This is fixed by doing `cargo install -f wasm-bindgen` manually

- openssl issue :

  try: `export NODE_OPTIONS=--openssl-legacy-provider` on unix or `set NODE_OPTIONS=--openssl-legacy-provider` on windows
