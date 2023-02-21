# API

The Ambient API is what developers use to interact with Ambient. It offers full access to Ambient's ECS, gameplay behavior, and more. At the time of writing, the Ambient API is only supported on the server, but we are actively working on bringing it to the client.

The easiest way to get started is by looking at some of the [examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples), which detail individual aspects of Ambient and its API. These can be combined to produce more complex functionality.

## Setting up your IDE

For Rust, we recommend using [Visual Studio Code](https://code.visualstudio.com/) with [rust-analyzer](https://rust-analyzer.github.io/), as described [here](https://code.visualstudio.com/docs/languages/rust).

Opening the `guest/rust` folder in VS Code or another Cargo/Rust-aware IDE will give you auto-completion and other related functionality.

## Reference documentation

The full API reference for Ambient can be found on [docs.rs](https://docs.rs/ambient_api).

Note that the published API may not be up to date with the latest Git commit of the runtime - if you are using bleeding-edge features, you will need to document the API yourself using `cargo doc -p ambient_api` in the `guest/rust` folder.
