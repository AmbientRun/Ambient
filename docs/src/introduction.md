# Introduction

Ambient is an open-source cross-platform runtime and platform for building, deploying and sharing high-performance multiplayer games on the web, desktop, and elsewhere. Powered by Rust, WebAssembly (WASM), and WebGPU, Ambient is cutting-edge while staying true to its goal: making game development both fun and accessible.

To set the scene: the Ambient runtime is an extensible multiplayer game engine with an in-game real-time database, automatic synchronization, Rust-inspired interoperable packages, an asset pipeline, WASM-powered isolation, PBR rendering, and more.

## Data

At the core of Ambient lies [an entity component system (ECS)](./reference/ecs.md) that forms the backbone of its data model - it's a real-time database for your game and everything running within it. _Entities_ contain _components_, which are typed pieces of data, and these are acted upon with _systems_.

Networked components are automatically synchronized to all clients, ensuring a consistent experience across all players; instead of grappling with complex networking intricacies, developers can focus on building their server and client-side logic.

In Ambient, everything is an entity with components, including the data of the runtime itself, ensuring that nothing is off-limits and all is accessible to developers. If you can see the data, you can use it.

## Packages

Experiences in Ambient are composed of [packages](./reference/package.md), which are bundles of code, assets and schema definitions. Packages can communicate with each other seamlessly through their schemas, allowing for structured, dynamic interoperability. Packages can be mixed and matched to create unique experiences, <!-- and they can be hot-reloaded on the fly, --> making Ambient the ultimate sandbox for multiplayer game development.

Packages can be deployed to the platform for other users to make use of, or to build on top of. Ambient offers a Rust-inspired package manager and tooling. Users specify dependencies in their package's manifest in a fashion similar to `Cargo.toml`. Rust programmers will feel at home, and non-Rust programmers will find the experience intuitive and easy to use.

## Assets

All assets, including code, are streamed to players when they connect to the server; users do not have to download anything to start playing games immediately<!--, and developers can swap out assets as required during development without having to restart the server -->.

<!-- With hot-reloading capabilities, you can make changes to your game logic and assets and see the results instantly, reducing development time and increasing productivity. This flexibility enables experimentation and rapid iteration, so that developers can focus on the most important part of game development: making fun games, quickly. -->

Ambient's [asset pipeline](./reference/asset_pipeline.md) supports a wide range of formats, including `.glb` and `.fbx`. The asset pipeline is flexible and can be extended to support additional formats as required. This approach ensures that developers can use their preferred tools and workflows without having to worry about compatibility issues.

## Code

[WebAssembly (WASM)](./reference/package.md#webassembly) is the secret sauce that enables Ambient's capabilities. Every package's code in Ambient operates within the confines of WebAssembly, ensuring a high level of isolation. Ambient pushes WASM to its absolute limits; on the web, WASM is used both to run the Ambient runtime and to execute user code, making it one of the most ambitious WASM projects to date.

Safety and stability are paramount. Thanks to the power of WebAssembly, code for Ambient runs in isolation. This means that if something within a package crashes, it won't bring down the entire program. Furthermore, the isolation provided by WebAssembly ensures that you can run untrusted code safely, enhancing security in multiplayer environments. This extends to embedding existing C/C++ libraries, which can be compiled to WebAssembly and used in Ambient packages.

## Rendering

At the heart of the Ambient renderer lies WebGPU, a cutting-edge technology that unleashes the potential of modern graphics hardware on the Web and beyond. By default, the renderer supports Physically Based Rendering (PBR) and offers advanced features such as cascading shadow maps and seamless instancing.

In future, the renderer will be made extensible, so that developers can define the visual style of their games as they see fit. This approach ensures that Ambient can be used to create a wide range of experiences, from realistic simulations to stylized games.

## Philosophy

Ambient's philosophy is based around flexibility and experimentation, empowering developers to push the boundaries in the ultimate game development sandbox. As Ambient develops, more and more functionality will be moved from the runtime to the realm of developers, ensuring that there are no limits on creativity.

Fundamentally, we are excited about the future of game creation. We would like to see completely new forms of games emerging, which we believe will only happen when game creation tools evolve into the next generation. We hope to usher in this new era with Ambient.

We want to forge a game creation platform that unleashes your creative energy and empowers you to share that fiery passion with the world - so that, one day, we can experience your dreams-made-real for ourselves.
