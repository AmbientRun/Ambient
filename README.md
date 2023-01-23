# Tilt Runtime

Tilt Runtime provides a programming environment for building high performance games and 3d applications. Some of the high level features are:

- Multiplayer by default: Everything built on Tilt can be used in multiplayer settings.
- WASM/WebAssembly powered; you can build modules for Tilt in a number of languages (though currently Rust is the only supported client language).
- Run everywhere: Tilt currently supports Window, Mac and Linux, but more platforms are under way.
- Data driven design: everything is built around a solid ECS core.
- GPU driven renderer: to give you maximum performance.
- Built in Rust
- Free and Open source

## Roadmap

| Feature | Status |
| ------- | ------ |
| ECS | ✅ |
| WASM API | ✅ |
| Gpu driven renderer | ✅ |
| Multiplayer/networking | ✅ |
| FBX & GLTF loading | ✅ |
| Physics (through Physx) | ✅ |
| Animations | ✅ |
| Skinmeshing | ✅ |
| Shadow maps | ✅ |
| Decals | ✅ |
| Run on Web | 🚧 |
| Client side scripting | 🚧 |
| UI | 🚧 |
| Custom shaders | 🚧 |
| Audio | 🚧 |

## Installing

You need [Rust](https://www.rust-lang.org/) to install the Tilt runtime. Then run:

```sh
cargo install tilt
```

### Dependencies: Linux/Ubuntu

```sh
apt-get install -y build-essential cmake pkg-config libfontconfig1-dev clang libasound2-dev ninja-build
```

## Running on headless Linux/Ubunutu

```sh
add-apt-repository ppa:oibaf/graphics-drivers -y
apt-get update
apt install -y libxcb-xfixes0-dev mesa-vulkan-drivers
```

## License (MIT)

Tilt is licensed under MIT. See [LICENSE.md](./LICENSE.md)
