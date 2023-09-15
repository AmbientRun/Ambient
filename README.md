# Ambient

[![Crates.io](https://img.shields.io/crates/v/ambient_api)](https://crates.io/crates/ambient_api)
[![docs.rs](https://img.shields.io/docsrs/ambient_api)](https://docs.rs/ambient_api)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/AmbientRun/Ambient#license)
[![Discord](https://img.shields.io/discord/894505972289134632)](https://discord.gg/PhmPn6m8Tw)

https://user-images.githubusercontent.com/707827/236472178-ed520f95-0e0a-434e-96f6-c66a6a96a8be.mp4

Ambient is a runtime for building high-performance multiplayer games and 3D applications, powered by WebAssembly, Rust and WebGPU.

See our [announcement blog post](https://www.ambient.run/post/introducing-ambient) for more details, or jump into our [documentation](https://ambientrun.github.io/Ambient/).

## Design principles

- **Seamless networking**: Ambient is both your server and client. All you need to do is to build your server and/or client-side logic: the runtime handles synchronization of data for you.
- **Isolation**: Packages that you build for Ambient are executed in isolation through the power of [WebAssembly](https://webassembly.org/) - so that if something crashes, it won’t take down your entire program. It also means that you can run untrusted code safely.
- **Data-oriented design**: The core data model of Ambient is an [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) which each WASM module can manipulate.
- **Language-agnostic**: You will be able to build Ambient modules in any language that can compile to WebAssembly. At present, Rust is the only supported language, but we are working on expanding to other languages.
- **Single executable**: Ambient is a single executable which can run on Windows, Mac and Linux. It can act as a server or as a client.
- **Interoperability**: Ambient allows you to define custom components and "concepts" (collections of components). As long as your Ambient packages use the same components and concepts, they will be able to share data and interoperate, even if they have no awareness of each other.
- **Asset pipeline and streaming**: Ambient has an [asset pipeline](https://ambientrun.github.io/Ambient/reference/asset_pipeline.html) that is capable of compiling multiple asset formats, including `.glb` and `.fbx`. The assets are always streamed over the network, so your clients will receive everything they need when they join.
- **Powerful renderer**: The Ambient renderer is GPU-driven, with both culling and level-of-detail switching being handled entirely by the GPU. By default, it uses [PBR](https://en.wikipedia.org/wiki/Physically_based_rendering). It also supports cascading shadow maps and instances everything that can be instanced.

See the [documentation](https://ambientrun.github.io/Ambient/) for a guide on how to get started, or browse the `guest/rust/examples` for the version of Ambient you're using. The `main` branch is a development branch and is likely incompatible with the latest released version of Ambient.

## Installing

Simply run:

```sh
cargo install ambient
```

This requires [Rust](https://www.rust-lang.org/) to be installed. You will also need to install the wasm32 toolchain: `rustup target add --toolchain stable wasm32-wasi`.

For alternative installation options, go to the [documentation on installing](https://ambientrun.github.io/Ambient/reference/advanced_installing.html).

## Roadmap

**_Note: Ambient is in an alpha stage and the API will be iterated on heavily. We are working towards a stable release._**

| Feature                 | Status | Notes                                                                                                                                                                                                                              |
| ----------------------- | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ECS                     | ✅     | Single-threaded.                                                                                                                                                                                                                   |
| WASM API                | ✅     | Rust is the only supported guest language right now, and WASM can be used on both the client and the server.                                                                                                                       |
| UI API                  | ✅     | Element UI, our React-inspired UI library, can be used in Rust WASM guests. See the [examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui).                                                            |
| Multiplayer/networking  | ✅     | Multiplayer is server-authoritative without any prediction or compensation. See [our documentation](https://ambientrun.github.io/Ambient/reference/networking.html).                                                               |
| GPU-driven renderer     | ✅     |                                                                                                                                                                                                                                    |
| FBX & glTF loading      | ✅     |                                                                                                                                                                                                                                    |
| Physics (through PhysX) | ✅     | Using PhysX 4.1. PhysX 5 support is tracked in [this issue](https://github.com/AmbientRun/Ambient/issues/155).                                                                                                                     |
| Animations              | ✅     |                                                                                                                                                                                                                                    |
| Skinmeshing             | ✅     |                                                                                                                                                                                                                                    |
| Shadow maps             | ✅     |                                                                                                                                                                                                                                    |
| Decals                  | ✅     |                                                                                                                                                                                                                                    |
| GPU culling and LoD     | ✅     |                                                                                                                                                                                                                                    |
| Multi-platform          | ✅     | Windows, Mac, and Linux so far. x86-64 and ARM64 are actively supported; other platforms may also work, but require testing.                                                                                                       |
| Audio                   | ✅     | Load sound, playback, looping, scale amp are supported. More featues on the way.                                                                                                                                                   |
| Run on Web              | 🚧     | See [this issue](https://github.com/AmbientRun/Ambient/issues/151).                                                                                                                                                                |
| Multithreading API      | 🚧     | Multithreading is already used internally, but we want to expose multithreading functionality within the WASM API. This may be explicit (i.e. task- or thread-spawning) or implicit (WASM modules being scheduled across threads). |
| Custom shaders          | 🚧     | Custom shaders are supported by the renderer, but are not yet exposed in the API. See [this issue](https://github.com/AmbientRun/Ambient/issues/98).                                                                               |
| Hot-reloading assets    | 🚧     | See [this issue](https://github.com/AmbientRun/Ambient/issues/12).                                                                                                                                                                 |
| ECS save/load           | 🚧     | For loading, [see this issue](https://github.com/AmbientRun/Ambient/issues/71).                                                                                                                                                    |
| IDE                     | 🚧     | [#694](https://github.com/AmbientRun/Ambient/issues/694)                                                                                                                                                                           |

In addition to the above the following areas have umbrella issues:
| Area           | Issue                                                    |
| -------------- | -------------------------------------------------------- |
| Networking     | [#764](https://github.com/AmbientRun/Ambient/issues/671) |
| Asset pipeline | [#764](https://github.com/AmbientRun/Ambient/issues/764) |

## Examples

Each example in the [examples](./guest/rust/examples/) directory can be run with Ambient as both client and server:

_(Note that ambient needs to match the version that the examples were built for, see ["Running examples"](https://ambientrun.github.io/Ambient/user/running_examples.html) in the docs)_

```
ambient run guest/rust/examples/controllers/first_person_camera
```

Every example can also be run server-only. To do so:

```
ambient serve guest/rust/examples/controllers/first_person_camera
```

This will start a server that other people, including yourself, can join. By default, the server will use the Ambient proxy to allow clients to join from outside your local network, and give you a URL to share with others:

```
ambient join proxy-eu.ambient.run:9176
```

Note that content is always streamed, so the only thing the joining user requires is Ambient itself to join the session.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
