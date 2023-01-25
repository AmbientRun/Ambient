# Tilt Runtime

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/TiltOrg/Tilt#license)
[![Discord](https://img.shields.io/discord/894505972289134632)](https://discord.gg/gYSM4tHZ)

Tilt Runtime provides a programming environment for building high performance games and 3d applications.

Our goal is to provide a free and Open source game development API/runtime which can be accessed from any language, which can be run on as many platforms as possible and which is multiplayer native. Since Tilt is powered by WASM, modules built on Tilt are always safe to run, both on your own game servers and on your clients machines.

We're also developing https://tilt.place, which will allow you to host and distribute your Tilt projects with ease.

See the [documentation](./docs) for a getting started guide.

## Installing

```sh
cargo install tilt
```

See [installing](./docs/src/installing.md) for more details.

## Roadmap & features

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
