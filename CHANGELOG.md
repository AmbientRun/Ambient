# Changelog

<!-- markdownlint-disable-file MD024 -->

This changelog is manually updated. While an effort will be made to keep the [Unreleased](#unreleased-yyyy-mm-dd) changes up to date, it may not be fully representative of the current state of the project.

<!-- If you are updating this file, make sure you copy the unreleased section and change the version and date. Do not remove it. -->

<!--
## Unreleased (YYYY-MM-DD)

### Added

#### Headline features

#### Other

#### Examples

### Changed

#### Breaking

#### Non-breaking

### Fixed

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

### Removed
-->

## Version 0.3.0-dev (YYYY-MM-DD)

### Added

#### Headline features

<!-- - **Client**: The client can now **run on the web**. -->

#### Other

#### Examples

### Changed

#### Breaking

- Locally-broadcasted messages can now choose to include the originating module in the broadcast; this is an additional boolean parameter to `ModuleMessage::send_local_broadcast` and `message::Target::LocalBroadcast`.
- `screen_to_world_direction` and `clip_space_ray` renamed. See [#410](https://github.com/AmbientRun/Ambient/issues/410)

#### Non-breaking

### Fixed

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

### Removed

## Version 0.2.1 (2023-05-06)

### Fixed

- **API**: The API documentation is now built only for the `wasm` target on `docs.rs`.

## Version 0.2.0 (2023-05-05)

### Added

#### Headline features

- **API**: Guest code can now **create and interact with UI**. See [the UI examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui).
- **API**: Guest code can now **run on the client**. See [the `clientside` example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/clientside).
- **API**: Clientside guest code can now play **basic audio**. See [the `pong` example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/games/pong).
- **Server**: By default, a proxy URL is generated for the server on startup. This can be used to access a running server from anywhere on the internet, making it easy to share your work with others. To turn this off, specify `--no-proxy` on the server command line.

#### Other

- **API**: Kinematic bodies are now exposed. This is used by the minigolf example to provide its moving obstacles.
- **API**: Added `physics::move_character` function to correctly move character controllers. This is used by the third-person camera example.
- **API**: `Uvec2`/`Uvec3`/`Uvec4`/`U8` can now be used for component values.
- **API**: A new `message` API has been added to allow for sending messages between client and server WASM, and from one WASM module to another. Messages are defined in `ambient.toml` and are structured. Message subscriptions return handles that can be used to cancel their subscriptions.
- **API**: A new `camera` API has been added on the client for operations that involve the camera, including `screen_ray` for calculating a ray from the camera through a screen position. Thanks to [@owenpalmer](https://github.com/owenpalmer) for implementing this in [#316](https://github.com/AmbientRun/Ambient/pull/316).
- **API**: A new `input` API has been added for retrieving input and manipulating the cursor (including changing its icon, visibility and lock state).
- **API**: `physics::{add_impulse, add_force_at_position, add_impulse_at_position, get_velocity_at_position}` have been added.
- **API**: Added `create_revolute_joint` to the `physics` API.
- **API**: Added a capsule concept with corresponding components.
- **API**: Several animation manipulation functions have been added to `entity` and `asset`. Thanks to [@devjobe](https://github.com/devjobe) for implementing this in [#362](https://github.com/AmbientRun/Ambient/pull/362).
- **Physics**: A `collider_loaded` component will now be automatically attached to an entity once its collider has finished loading.
- **Client**: The client's window title is now automatically changed to the name of the project running on the server. Thanks to [@MavethGH](https://github.com/MavethGH) for implementing this in [#178](https://github.com/AmbientRun/Ambient/pull/178).
- **Client**: Added a basic headless mode to enable automatic CI testing of projects.
- **Client**: Added `Dump UI World` button to inspect the state of the UI. Thanks to [@owenpalmer](https://github.com/owenpalmer) for implementing this in [#216](https://github.com/AmbientRun/Ambient/pull/216).

#### Examples

- A suite of UI examples have been added to demonstrate how to use the UI in guest code.
- The `clientside` example shows how to use clientside WASM.
- The `messaging` example shows how to message the server from the client and vice versa, and how to message another module with both broadcasts and directed messages.
- The `pong` example implements a basic version of Pong to demonstrate a basic multiplayer game.
- The `fog` example shows how to configure fog in the renderer for more atmospheric scenes.
- The `first_person_camera` example shows how to implement a first-person camera.
- The `music_sequencer` example shows how to use the audio and UI API to build a basic music sequencer.
- The `decals` example shows how to use decals to add detail to a scene. Thanks to [@kevzettler](https://github.com/kevzettler) for implementing this in [#347](https://github.com/AmbientRun/Ambient/pull/347).

### Changed

#### Breaking

- **Client**: `--debug` is now `--debugger`, but it can also be accessed through the `AMBIENT_DEBUGGER` env variable.
- **API**: The `Cargo.toml` has changed to enable clientside builds. Please look at the examples to see how to update your `Cargo.toml` appropriately.
- **API**: `ChangeQuery` has been split into `UntrackedChangeQuery` and `ChangeQuery` to ensure that `track_change` is called before the query is built.
- **API**: `asset_url` has moved to `asset::url`.
- **API**: `EventResult` and `EventOk` have been renamed to `ResultEmpty` and `OkEmpty` to better clarify their purpose.
- **API**: The physics API has been revamped to better encode the physics engine's capabilities.
  - `physics::apply_force` is now `physics::add_force`.
  - `physics::explode_bomb` is now `physics::add_radial_impulse`, and takes a `FalloffRadius` enum.
- **API**: All input functionality has moved to `input` on the clientside.
- **API**: The `lookat_center` component has been renamed to `lookat_target`.
- **Physics**: Convex shapes are now used if a body is neither static or kinematic.

#### Non-breaking

- **Ambient**: Ambient is now dual-licensed MIT/Apache2, in accordance with the rest of the Rust ecosystem.
- **Ambient**: The default logging settings now better communicate what Ambient is doing at any given moment.
- **Project**: Concept definitions in projects now support namespaces. Thanks to [@ArberSephirotheca](https://github.com/ArberSephirotheca) for implementing this in [#212](https://github.com/AmbientRun/Ambient/pull/212).
- **API**: Concepts now include the components they use in their doc comments.
- **API**: `#[main]`-attributed functions no longer have to be `async` or return a `Result`.
- **API**: `#[main]`-attributed functions, `on`, `once`, `Query::bind` and `run_async` can now return a `Result` or nothing.
- **Project**: Project manifests can now be split into multiple files using `includes`.

### Fixed

- **Ambient**: Various stability and performance fixes.
- **Ambient**: Added attributions for external code.
- **Ambient**: Typo fixes. Thanks for the following!
  - [#159: fix: docs broken links to gh-pages](https://github.com/AmbientRun/Ambient/pull/159) by [@daniellavoie](https://github.com/daniellavoie)
  - [#172: chore: fix typo in gpu.rs](https://github.com/AmbientRun/Ambient/pull/172) by [@eltociear](https://github.com/eltociear)
- **Examples**: The Minigolf example now has several gameplay tweaks (including camera movement on right-click) to improve the experience.
- **Examples**: The examples no longer occasionally use non-one alpha colours, which led to them rendering black objects.
- **Server**: The server no longer shuts down automatically after a period of inactivity.
- **ECS**: A bug with ECS component versioning that led to certain components not updating has been fixed. Fixes [#113](https://github.com/AmbientRun/Ambient/issues/113).
- **Networking**: Various optimizations have been made to networking and the ECS to reduce unnecessary network traffic.

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

- **CI**: Linux CI builds now output the tree of their target to assist in debugging CI cache blow-up. Thanks to [@daniellavoie](https://github.com/daniellavoie) for implementing this in [#170](https://github.com/AmbientRun/Ambient/pull/170).
- **ECS**: `Entity::assert_all` can be used to ensure all components for an `Entity` on the host have an attribute. Thanks to [@MavethGH](https://github.com/MavethGH) for implementing this in [#211](https://github.com/AmbientRun/Ambient/pull/211).
- **App**: `ambient new` uses the correct path for relative API when creating a project in `guest/rust/examples`. Thanks to [@owenpalmer](https://github.com/owenpalmer) for implementing this in [#218](https://github.com/AmbientRun/Ambient/pull/218).
- **Ambient**: The presentation of the license in the repository was improved. Thanks to [@C-BJ](https://github.com/C-BJ) for [#201](https://github.com/AmbientRun/Ambient/pull/201) and [#203](https://github.com/AmbientRun/Ambient/pull/203).
- **Ambient**: The book and build CI workflows now only run when relevant files are updated. Thanks to [@C-BJ](https://github.com/C-BJ) for implementing this in [#202](https://github.com/AmbientRun/Ambient/pull/202).
- **Audio**: The audio asset pipeline now uses Rust libraries for re-encoding files, instead of shelling out to ffmpeg. Thanks to [@marceline-cramer](https://github.com/marceline-cramer) for implementing this in [#317](https://github.com/AmbientRun/Ambient/pull/317).
- **Rendering**: Ambient now runs on wgpu 0.16, improving compatibility and providing access to new features. Thanks to [@kevzettler](https://github.com/kevzettler) for implementing this in [#308](https://github.com/AmbientRun/Ambient/pull/308).
- **Campfire**: The internal development tool Campfire can now automatically check release-readiness. Thanks to [@kevzettler](https://github.com/kevzettler) for implementing this in [#356](https://github.com/AmbientRun/Ambient/pull/356).

### Removed

- **API**: `player_camera` has been removed, and the components it instantiated are now directly exposed. See the `multiplayer` example to see what's changed.
- **API**: Events have been removed and replaced with the more general-purpose `message` API.

## Version 0.1.1 (2023-02-22)

### Added

- A [minigolf example](guest/rust/examples/minigolf) by [SK83RJOSH](https://github.com/SK83RJOSH).
- Examples are now bundled into a downloadable `examples.zip` for each release.

### Fixed

- macOS ARM64 builds are now available after enabling the execution of unsigned executable memory (as required for wasmtime execution).
- The debugging configuration for VSCode was updated to use the new CLI.
- Minor documentation updates.

## Version 0.1.0 (2023-02-22)

Initial release. See the [announcement blog post](https://www.ambient.run/post/introducing-ambient).
