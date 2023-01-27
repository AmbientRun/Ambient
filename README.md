# Tilt Runtime

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/TiltOrg/Tilt#license)
[![Discord](https://img.shields.io/discord/894505972289134632)](https://discord.gg/gYSM4tHZ)

Tilt is a WebAssembly runtime for building high-performance online games and 3d applications.

Features:
- **Seamless networking**: Tilt is both your server and client. You just need to build your server and/or client side logic, the runtime handles synchronization of data for you.
- **Isolation**: Packages you build for Tilt are executed in isolation, which means that even if something crashes, it won’t take down your entire program. It also means that you can run untrusted code safely.
- **Data oriented design**: The core data model of Tilt is an ECS which each “process” can manipulate.
- **Multi-lingual**: You will be able to build Tilt modules in any language which compiles to WebAssembly. Rust is the only supported language yet, but we’re working on expanding to other languages.
- **Single executable**: The Tilt Runtime is a single executable which runs on Windows, Mac and Linux. It can act as a server, or as a client, in which case it renders your game world for you.
- **Interoperability**: We’ve designed a way for you to define “component schemas”, which makes it possible to share data between Tilt modules, even if they don’t explicitly know about each other.
- **Asset pipeline & streaming**: The runtime supports building and loading multiple file formats, such as .glb and .fbx. The assets are always streamed over the network.
- **Powerful renderer**: The Tilt renderer is “gpu driven”, with culling and lodding happening on the gpu side. It’s PBR by default, supports cascading shadow maps, and instances everything that can be instanced.

We're also developing https://tilt.place, which will allow you to host and distribute your Tilt projects with ease.

See the [documentation](./docs) for a getting started guide.

## Installing

```sh
cargo install tilt
```

See [installing](./docs/src/installing.md) for more details.

## Roadmap

***Note; Tilt is in an alpha stage and the API will still be iterated on heavily. We're working towards a stable release.***

| Feature | Status | Notes |
| ------- | ------ | ----- |
| ECS | ✅ |
| WASM API | ✅ | *Rust is the only supported client language right now* |
| Multiplayer/networking | ✅ |
| Gpu driven renderer | ✅ |
| FBX & GLTF loading | ✅ |
| Physics (through Physx) | ✅ |
| Animations | ✅ |
| Skinmeshing | ✅ |
| Shadow maps | ✅ |
| Decals | ✅ |
| GPU culling and lodding | ✅ |
| Multi platform | ✅ | *Windows, Mac, Linux so far* |
| Run on Web | 🚧 |
| Client side API | 🚧 |
| Multithreading API | 🚧 | *Multithreading is used internally already, but we want to expose an API on the WASM side* |
| UI API | 🚧 | *A React-like UI library already exists in the repo, and we're working on exposing it through the WASM API* |
| Custom shaders | 🚧 | *Custom shaders are supported by the renderer, but not yet exposed in the API* |
| Hot-reloading | 🚧 |
| Audio | 🚧 |
| Save/load ECS | 🚧 |

## Examples

Each example in the [examples](./examples/) directory can be run with e.g. `tilt run --project_path=examples/hello_world`.

## Contributing

We welcome community contributions to this project.

Please talk with us on Discord beforehand if you'd like to contribute a larger piece of work.

## License (MIT)

Tilt is licensed under MIT. See [LICENSE.md](./LICENSE.md)
