# Ambient

[![Crates.io](https://img.shields.io/crates/v/ambient_api)](https://crates.io/crates/ambient_api)
[![docs.rs](https://img.shields.io/docsrs/ambient_api)](https://docs.rs/ambient_api)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/AmbientRun/Ambient#license)
[![Discord](https://img.shields.io/discord/894505972289134632)](https://discord.gg/gYSM4tHZ)

Ambient is a runtime for building high-performance multiplayer games and 3D applications, powered by WebAssembly, Rust and WebGPU.

[Announcement blog post](https://www.ambient.run/post/introducing-ambient)

## Features

- **Seamless networking**: Ambient is both your server and client. All you need to do is to build your server and/or client-side logic: the runtime handles synchronization of data for you.
- **Isolation**: Projects you build for Ambient are executed in isolation through the power of [WebAssembly](https://webassembly.org/) - so that if something crashes, it won’t take down your entire program. It also means that you can run untrusted code safely.
- **Data-oriented design**: The core data model of Ambient is an [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) which each WASM module can manipulate.
- **Multilingual**: You will be able to build Ambient modules in any language that can compile to WebAssembly. At present, Rust is the only supported language, but we are working on expanding to other languages.
- **Single executable**: Ambient is a single executable which can run on Windows, Mac and Linux. It can act as a server or as a client.
- **Interoperability**: Ambient allows you to define custom components and "concepts" (collections of components). As long as your Ambient projects use the same components and concepts, they will be able to share data and interoperate, even if they have no awareness of each other.
- **Asset pipeline and streaming**: Ambient has an [asset pipeline](https://ambientrun.github.io/Ambient/asset_pipeline.html) that is capable of compiling multiple asset formats, including `.glb` and `.fbx`. The assets are always streamed over the network, so your clients will receive everything they need when they join.
- **Powerful renderer**: The Ambient renderer is GPU-driven, with both culling and level-of-detail switching being handled entirely by the GPU. By default, it uses [PBR](https://en.wikipedia.org/wiki/Physically_based_rendering). It also supports cascading shadow maps and instances everything that can be instanced.

See the [documentation](https://ambientrun.github.io/Ambient/) for a getting started guide, or browse the [examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples).

## Installing

The easiest way to get Ambient is by downloading the latest release [here](https://github.com/AmbientRun/Ambient/releases).

For alternative installation options, go the [documentation on installing](https://ambientrun.github.io/Ambient/installing.html).

## Roadmap

**_Note: Ambient is in an alpha stage and the API will be iterated on heavily. We are working towards a stable release._**

| Feature                 | Status | Notes                                                                                                                |
| ----------------------- | ------ | -------------------------------------------------------------------------------------------------------------------- |
| ECS                     | ✅     |                                                                                                                      |
| WASM API                | ✅     | _Rust is the only supported client language right now._                                                              |
| Multiplayer/networking  | ✅     |                                                                                                                      |
| GPU-driven renderer     | ✅     |                                                                                                                      |
| FBX & GLTF loading      | ✅     |                                                                                                                      |
| Physics (through PhysX) | ✅     |                                                                                                                      |
| Animations              | ✅     |                                                                                                                      |
| Skinmeshing             | ✅     |                                                                                                                      |
| Shadow maps             | ✅     |                                                                                                                      |
| Decals                  | ✅     |                                                                                                                      |
| GPU culling and LoD     | ✅     |                                                                                                                      |
| Multi-platform          | ✅     | _Windows, Mac and Linux so far._                                                                                     |
| Run on Web              | 🚧     |                                                                                                                      |
| Client-side API         | 🚧     |                                                                                                                      |
| Multithreading API      | 🚧     | _Multithreading is used internally already, but we want to expose multithreading functionality within the WASM API._ |
| UI API                  | 🚧     | _A React-like UI library already exists in the repo, and we're working on exposing it through the WASM API._         |
| Custom shaders          | 🚧     | _Custom shaders are supported by the renderer, but not yet exposed in the API._                                      |
| Hot-reloading assets    | 🚧     |                                                                                                                      |
| Audio                   | 🚧     | Audio is supported, but not currently exposed.                                                                       |
| ECS save/load           | 🚧     |                                                                                                                      |

## Examples

Each example in the [examples](./guest/rust/examples/) directory can be run with Ambient:

- `cd guest/rust/examples/tictactoe`
- `ambient run`

Every example can also be run in multiplayer mode, but may not have any multiplayer-specific behaviour. To do so:

- `cd guest/rust/examples/tictactoe`
- `ambient serve`

This will start a server that other people can join:

- `ambient join [IP_OF_SERVER]`

Note that content is always streamed so the only thing the joining user requires is Ambient itself to join the session.

## Contributing

We welcome community contributions to this project.

Please talk with us on Discord beforehand if you'd like to contribute a larger piece of work.

## License (MIT)

Ambient is licensed under MIT. See the [LICENSE](./LICENSE.md).
