# Introduction

Ambient is a WebAssembly runtime for building high-performance online games and 3D applications.

## Features

- **Seamless networking**: Ambient is both your server and client. All you need to do is to build your server and/or client-side logic: the runtime handles synchronization of data for you.
- **Isolation**: Projects you build for Ambient are executed in isolation through the power of [WebAssembly](https://webassembly.org/) - so that if something crashes, it won’t take down your entire program. It also means that you can run untrusted code safely.
- **Data-oriented design**: The core data model of Ambient is an [entity component system](https://en.wikipedia.org/wiki/Entity_component_system) which each WASM module can manipulate.
- **Language-agnostic**: You will be able to build Ambient modules in any language that can compile to WebAssembly. At present, Rust is the only supported language, but we are working on expanding to other languages.
- **Single executable**: Ambient is a single executable which can run on Windows, Mac and Linux. It can act as a server or as a client.
- **Interoperability**: Ambient allows you to define custom components and "concepts" (collections of components). As long as your Ambient projects use the same components and concepts, they will be able to share data and interoperate, even if they have no awareness of each other.
- **Asset pipeline and streaming**: Ambient has an [asset pipeline](https://ambientrun.github.io/Ambient/asset_pipeline.html) that is capable of compiling multiple asset formats, including `.glb` and `.fbx`. The assets are always streamed over the network, so your clients will receive everything they need when they join.
- **Powerful renderer**: The Ambient renderer is GPU-driven, with both culling and level-of-detail switching being handled entirely by the GPU. By default, it uses [PBR](https://en.wikipedia.org/wiki/Physically_based_rendering). It also supports cascading shadow maps and instances everything that can be instanced.

See the [documentation](https://ambientrun.github.io/Ambient/) for a getting started guide, or browse the [examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples).
