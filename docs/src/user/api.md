# API

The Ambient API is what developers use to interact with Ambient. It offers full access to Ambient's ECS, gameplay behavior, and more.

The easiest way to get started is by looking at some of the [examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples), which detail individual aspects of Ambient and its API. These can be combined to produce more complex functionality.

Opening the `guest/rust` folder in VS Code or another Cargo/Rust-aware IDE will give you auto-completion and other related functionality.

## Reference documentation

The full API reference for Ambient can be found on [docs.rs](https://docs.rs/ambient_api).

Note that the published API may not be up to date with the latest Git commit of the runtime - if you are using bleeding-edge features, you will need to document the API yourself using `cargo doc -p ambient_api` in the `guest/rust` folder.

## Should my code go on the client or the server?

The Ambient API is split into two parts: the client and the server. The client is the code that runs on the player's machine, and the server is the code that runs on the host's machine. The client is responsible for rendering the game, and for sending input to the server. The server is responsible for running the game simulation, and for sending the client information about the game state.

When you create a project, both `client` and `server` modules are created. You can put code in either of these modules, and it will be run on the client or the server, respectively. In general, code that runs on the server should be authoritative, and code that runs on the client should be visual. What the server says should be the source of truth for all players.

The ECS can be used to synchronize state between the server and the client. Both the client and the server have the same ECS, but components with the `Networked` attribute will be synchronized from the server to the client. The client can make its own changes to the ECS, including adding and modifying components, but any modified components will be overridden by the server's version when the server sends an update for those components.

Additionally, both the client and the server can send structured messages to each other to communicate information that can't be represented in the ECS. For more information on this, see the [project documentation](../reference/project.md#messages--messages).

Deciding where your code should go is important to making the most of Ambient, and it's not always obvious. Here are some guidelines:

If you are doing any of the following, your code should go on the client:

- Rendering UI
- Visual changes that should only be visible to the player
- Capturing input
- Playing sounds
- Predicting the game state for better user experience
- Visual effects that don't need to be replicated exactly (particle systems, etc)

If you are doing any of the following, your code should go on the server:

- Moving a character
- Calculating damage
- Spawning or updating entities
- Changing the game state
- Communicating with external services
- Anything that should be authoritative
- Anything that should be hidden from the player

If you are doing any of the following, your code could go on either the client or the server, or be shared between them:

- Shared calculations (e.g. determining the color of a player's nameplate from the player's name)
- Shared data structures
- Shared constants
- Shared utility functions
- Shared types

Consider looking at the game examples for more information on how to structure your code.
