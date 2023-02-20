# Ambient Runtime

![Crates.io](https://img.shields.io/crates/v/ambient_api)
![docs.rs](https://img.shields.io/docsrs/ambient_api)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/AmbientOrg/Ambient#license)
[![Discord](https://img.shields.io/discord/894505972289134632)](https://discord.gg/gYSM4tHZ)

Ambient is a WebAssembly runtime for building high-performance online games and 3d applications.

Features:

- **Seamless networking**: Ambient is both your server and client. You just need to build your server and/or client side logic, the runtime handles synchronization of data for you.
- **Isolation**: Packages you build for Ambient are executed in isolation, which means that even if something crashes, it won’t take down your entire program. It also means that you can run untrusted code safely.
- **Data oriented design**: The core data model of Ambient is an ECS which each “process” can manipulate.
- **Multi-lingual**: You will be able to build Ambient modules in any language which compiles to WebAssembly. Rust is the only supported language yet, but we’re working on expanding to other languages.
- **Single executable**: The Ambient Runtime is a single executable which runs on Windows, Mac and Linux. It can act as a server, or as a client, in which case it renders your game world for you.
- **Interoperability**: We’ve designed a way for you to define “component schemas”, which makes it possible to share data between Ambient modules, even if they don’t explicitly know about each other.
- **Asset pipeline & streaming**: The runtime supports building and loading multiple file formats, such as .glb and .fbx. The assets are always streamed over the network.
- **Powerful renderer**: The Ambient renderer is “gpu driven”, with culling and lodding happening on the gpu side. It’s PBR by default, supports cascading shadow maps, and instances everything that can be instanced.

See the [documentation](https://ambientorg.github.io/Ambient/) for a getting started guide, or browse the [examples](https://github.com/AmbientOrg/Ambient/tree/main/guest/rust/examples).

## Installing

```sh
cargo install --git https://github.com/AmbientOrg/Ambient.git
```

See [installing](https://ambientorg.github.io/Ambient/installing.html) for more details.

## Roadmap

**_Note; Ambient is in an alpha stage and the API will still be iterated on heavily. We're working towards a stable release._**

| Feature                 | Status | Notes                                                                                                       |
| ----------------------- | ------ | ----------------------------------------------------------------------------------------------------------- |
| ECS                     | ✅     |
| WASM API                | ✅     | _Rust is the only supported client language right now_                                                      |
| Multiplayer/networking  | ✅     |
| Gpu driven renderer     | ✅     |
| FBX & GLTF loading      | ✅     |
| Physics (through Physx) | ✅     |
| Animations              | ✅     |
| Skinmeshing             | ✅     |
| Shadow maps             | ✅     |
| Decals                  | ✅     |
| GPU culling and lodding | ✅     |
| Multi platform          | ✅     | _Windows, Mac, Linux so far_                                                                                |
| Run on Web              | 🚧     |
| Client side API         | 🚧     |
| Multithreading API      | 🚧     | _Multithreading is used internally already, but we want to expose an API on the WASM side_                  |
| UI API                  | 🚧     | _A React-like UI library already exists in the repo, and we're working on exposing it through the WASM API_ |
| Custom shaders          | 🚧     | _Custom shaders are supported by the renderer, but not yet exposed in the API_                              |
| Hot-reloading           | 🚧     |
| Audio                   | 🚧     |
| Save/load ECS           | 🚧     |

## Examples

Each example in the [examples](./guest/rust/examples/) directory can be run with the ambient runtime: `cd guest/rust/examples/tictactoe` and then `ambient run`.
They can also be run in multiplayer mode; `cd examples/tictactoe` and then `ambient serve` to start a serve. Anyone can now join
with `ambient join [IP_OF_SERVER]`. Note that content is always streamed so the user joining only needs the ambient cli to join the session.

## Contributing

We welcome community contributions to this project.

Please talk with us on Discord beforehand if you'd like to contribute a larger piece of work.

## License (MIT)

Ambient is licensed under MIT. See [LICENSE.md](./LICENSE.md)
