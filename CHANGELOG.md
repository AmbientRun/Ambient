# Changelog

<!-- markdownlint-disable-file MD024 -->

This changelog is manually updated. While an effort will be made to keep the [Unreleased](#unreleased-yyyy-mm-dd) changes up to date, it may not be fully representative of the current state of the project.

<!-- If you are updating this file, make sure you copy the unreleased section and change the version and date. Do not remove it. -->

<!--
## Unreleased (YYYY-MM-DD)

### Added

#### Headline features

#### Other

### Changed

#### Breaking

#### Non-breaking

### Fixed

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

### Removed
-->

## Unreleased (YYYY-MM-DD)

### Added

#### Headline features

#### Other

- **Package/character_movement**: CharacterMovement concept added
- **Assets**: Added support for animations and skinning for assimp
- **Examples**: Added assimp example
- **Examples**: Added benchmark/animations example

### Changed

#### Breaking

#### Non-breaking

### Fixed

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

### Removed

## Version 0.3.0 (2023-10-04)

This release involved a great many changes, including porting Ambient to the web, adding support for deployments, many API changes, and more. Not all changes may be reflected in this changelog; please let us know if you find any omissions.

### Added

#### Headline features

- **Client**: The client can now **run on the web**.
- **Deploy**: The `ambient deploy` command can now be used to deploy a package to Ambient runtime services.
- **Audio**: Spatial audio is now supported for 3D sounds. See the [physics example](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/examples/physics/basics/src/client.rs) and [first_person_camera example](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/examples/controllers/first_person_camera/src/client.rs)
- **Networking**: The networking protocol now supports WebTransport for the web client.
- **Rendering**: Procedural meshes, textures, samplers and materials are now supported on the client. See the [procedural generation example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/rendering/procedural_generation).
- **Semantics**: A semantic system to connect packages (previously projects) has been added. This enables dependencies, enums and more. See the breaking changes for more details.

#### Other

- **UI**: Added a new `ImageFromUrl` element, which can load images from assets or URLs. It also supports rounded corners, borders and a fallback background color. See the [image example](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/examples/ui/image/src/client.rs) for more details.
- **Rendering**: Added a `torus` primitive. Thanks to [@mebyz](https://github.com/mebyz) for implementing this in [#376](https://github.com/AmbientRun/Ambient/pull/376)!
- **Physics**: Add `set_character_controller_position` to the `physics` API. Thanks to [@devjobe](https://github.com/devjobe) for implementing this in [#398](https://github.com/AmbientRun/Ambient/pull/398).
- **ECS**: `Duration` is now a supported primitive type.
- **ECS**: All integer types from 8-bit to 64-bit are now supported as component types, including signed and unsigned variants. Additionally, all signed and unsigned integer vector types are now supported. This includes `U16`, `IVec2`, `UVec3`, etc.
- **Docs**: The IDE documentation has been improved, including information on how to set up Emacs for Ambient development (thanks to [@kevzettler](https://github.com/kevzettler) in [#505](https://github.com/AmbientRun/Ambient/pull/505)).
- **Assets**: `ambient assets import` can be used to import assets one by one. This will create or modify the `pipeline.toml` file for you.
- **Camera**: Added `camera::get_active` to get the active camera.
- **Client**: When using the native client, you can now use `--window-x` and `--window-y` to specify the window position, as well as `--window-width` and `--window-height` to specify the window size.
- **Packages**: A set of standard packages have been added. Explore `guest/rust/packages` to find them. These include a character controller, a focus-grabber, a package manager, and more.
- **Packages**: Several sample games have been added, including `tangent`, `arkanoid`, `pong`, and more.
- **Examples**: Examples have been added for new functionality, including `clock`, `audio_ctrl`, `dependencies`, and more.
- **Debugging**: Sentry has been added to the client and server to automatically report crashes. This can be disabled in Ambient's `settings.toml`.

#### Breaking

- **Examples**: Examples have been rearranged to be more discoverable. They are now grouped by category.
- **Project**: Projects have been renamed to Packages; see the [package documentation](https://ambientrun.github.io/Ambient/reference/package.html) for details.
- **Package**: As mentioned above, a new package semantic system has been added. This comes with several breaking changes:

  - In your Rust code:

    - The modules generated by the Ambient macro are now different in shape. They are namespaced, with each item being present in the namespace as a separate module within the `packages` module, where your package is `this`. For example:
      - `components::my_component` -> `packages::this::components::my_component`
      - `ambient_api::components::core::app::main_scene` -> `ambient_api::core::app::components::main_scene`
      - a dependency `cool_dependency = { path = "somewhere" }` -> `packages::cool_dependency::components::my_component` (i.e. the name used for the import is used, not the package ID)
    - `asset::url` has been removed; instead, each package now introduces an `assets` module, allowing you to access that package's assets directly. For example:
      - `asset::url("assets/Teapot.glb").unwrap()` -> `packages::ambient_example_asset_loading::assets::url("Teapot.glb")`
    - UI layout components now use enums to specify their alignment. For example:
      - `with_default(fit_vertical_none())` -> `with(fit_vertical(), Fit::None)`
    - Messages now preserve the order of their fields.
      - If your fields are `a, c, b`, they will be in that order in the generated code.
      - Previously, they were sorted alphabetically.

  - In your `ambient.toml`:

    - Specifying a `version` is now mandatory.
    - Specifying a `content` is now mandatory.
    - Messages now have PascalCase IDs:
      - `messages.set_controller` -> `messages.SetController`
    - Enums are now supported; they are defined as follows, and can be used anywhere a primitive type can be used, including components and messages.

      ```toml
      [enums.Layout]
      description = "The type of the layout to use."
      [enums.Layout.members]
      Flow = "Bottom-up flow layout."
      Dock = "Top-down dock layout."
      Bookcase = "Min-max bookcase layout."
      WidthToChildren = "Width to children."

      [components]
      layout = { type = "Layout" }
      ```

    - Packages can now depend on other packages, allowing for access to their components, messages, etc. This is done by adding a `dependencies` section to your package. For example:

      ```toml
      [dependencies]
      my_cool_package = { path = "cool_package" }
      ```

      This will allow you to access the components, messages, etc. of the `my_cool_package` package from your package. The name that you use to import the package will be used as the namespace, not the package's original ID.

- **Components**: Several "tag components" have been prefixed with `is_` to indicate their tag nature. These include:

  - `core::animation::animation_player` -> `core::animation::is_animation_player`
  - `core::audio::audio_player` -> `core::audio::is_audio_player`
  - `core::audio::spatial_audio_player` -> `core::audio::is_spatial_audio_player`
  - `core::layout::screen` -> `core::layout::is_screen`
  - `core::network::persistent_resources` -> `core::network::is_persistent_resources`
  - `core::network::synced_resources` -> `core::network::is_synced_resources`
  - `core::player::player` -> `core::player::is_player`
  - `core::wasm::module` -> `core::wasm::is_module`
  - `core::wasm::module_on_server` -> `core::wasm::is_module_on_server`

- **API**: Locally-broadcasted messages can now choose to include the originating module in the broadcast; this is an additional boolean parameter to `ModuleMessage::send_local_broadcast` and `message::Target::LocalBroadcast`.
- **Camera**: Renamed `screen_to_world_direction` to `screen_position_to_world_ray` and `clip_space_ray` to `clip_position_to_world_ray`. See [#410](https://github.com/AmbientRun/Ambient/issues/410).
- **Package**: `type = { type = "Vec3" }` is no longer valid syntax in `ambient.toml`. Only `type = "Vec3"` and `type = { type = "Vec", element-type = "Vec3" }` are valid.
- **Physics**: Renamed the `visualizing` component to `visualize_collider`.
- **Animation**: The animation system has been reworked. See the [animation documentation](https://ambientrun.github.io/Ambient/reference/animations.html) for details. Thanks to [@devjobe](https://github.com/devjobe) for laying the foundation for this!
- **Physics**: Renamed `box_collider` to `cube_collider`.
- **API**: The `time` function has been split into `game_time` and `epoch_time`. The `dtime` component has been renamed to `delta_time`. The `frametime` function has been renamed to `delta_time`.
- **Assets**: Asset pipelines now use TOML instead of JSON. Use the `ambient assets migrate-pipelines-toml` command to migrate. (Note that this command will be removed in the next release.)
- **API**: Removed `Entity::with_default` due to its confusing behaviour (Rust defaults are not necessarily the same as component or concept defaults). You will now have to explicitly specify a value for each component.
- **API**: `entity::wait_for_component` is now marked as `must_use` to ensure users consider the possibility of the entity being despawned.
- **Elements**: All hooks are now free functions (i.e. `use_state(hooks, ..)` instead of `hooks.use_state(..)`)
- **UI**: Focus is now global across different packages, including the removal of the `FocusRoot` element component.
- **Hierarchies**: The `children` component is now automatically derived from `parent` components (unless the user opts out of this). The `children` component is also no longer networked, as it is automatically derived on the client.
- **Concepts**: Concept code generation has been changed to generate `structs` instead, as well as adding support for optional components. See the documentation for more information.

#### Non-breaking

- **Logging**: The logging output levels have been tweaked to better communicate the state of the system at any given time.
- **Debugger**: The debugger has been improved with a resizable sidebar, a scrollable view, and a component filter.
- **Animation**: The animation graph is now executed on the server as well.
- **CLI**: The `ambient new` command now takes several parameters to customize the resulting generation.
- **Rendering**: The renderer now runs using the direct pipeline on all architectures. This is temporary until platform-specific issues with the multi-draw indirect pipeline are addressed.

### Fixed

- **Rendering**: Skinned meshes will no longer be corrupted when there is more than one skinned mesh in a mesh buffer.
- **UI**: `TextEditor` will no longer capture input even when it is not visible.
- **Rendering**: Decals now render more consistently.
- **API**: `entity::wait_for_component` will now exit if the entity is despawned.
- **API**: The `message::Source` methods no longer consume the source when returning their data.
- **Rendering**: Lines with a `from` located after a `to` on the X-dimension will now render correctly.
- **API**: The `entity::mutate_component` documentation now refers to the correct parameter. Thanks to [@aldzban](https://github.com/aldzban) for fixing this in [#482](https://github.com/AmbientRun/Ambient/pull/482).
- **UI**: The `ScrollArea` now has a scroll bar.
- **Input**: Input is now cleared when the window loses focus, preventing "stuck input" bugs.
- **UI**: Layout-related properties, like alignment and fit, did not work correctly for certain values. This has been fixed with the introduction of enums.
- **Player**: Player entities are now recursively despawned when disconnecting.
- **Build**: Rust compilation errors are now more readable with more colors and fewer unused warnings.
- **ECS**: The `transformable` concept now includes `local_to_world` to ensure that the world transform is always available.
- **Physics**: The `physics::raycast[_first]` functions will now validate the direction to ensure that they are non-zero, non-NaN and normalized.
- **Rendering**: Removing the `outline_recursive` component from a entity will now remove the outline from its children as well.
- **API**: The `ambient_ui` prelude (and the `ambient_api` prelude, by extension) no longer glob-imports components into the global namespace. This means that you will need to import components explicitly.
- **Messaging**: Messages without empty fields now generate a unit struct, instead of a struct with no fields. That is, they generate `struct MyMessage;` instead of `struct MyMessage {}`.
- **Physics**: Child collision volumes are now automatically updated when their parent's transform changes. Thanks to [@kevzettler](https://github.com/kevzettler) for fixing this in [#885](https://github.com/AmbientRun/Ambient/pull/885)!

### Community PRs to internals

These PRs are not directly user-facing, but improve the development experience. They're just as appreciated!

### Changed

- `glam` was updated to 0.24. Thanks to [@devjobe](https://github.com/devjobe) for implementing this in [#434](https://github.com/AmbientRun/Ambient/pull/434).

### Removed

## Version 0.2.1 (2023-05-06)

### Fixed

- **API**: The API documentation is now built only for the `wasm` target on `docs.rs`.

## Version 0.2.0 (2023-05-05)

### Added

#### Headline features

- **API**: Guest code can now **create and interact with UI**. See [the UI examples](https://github.com/AmbientRun/Ambient/tree/v0.2.0/guest/rust/examples/ui).
- **API**: Guest code can now **run on the client**. See [the `clientside` example](https://github.com/AmbientRun/Ambient/tree/v0.2.0/guest/rust/examples/basics/clientside).
- **API**: Clientside guest code can now play **basic audio**. See [the `pong` example](https://github.com/AmbientRun/Ambient/tree/v0.2.0/guest/rust/examples/games/pong).
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

- A [minigolf example](https://github.com/AmbientRun/Ambient/tree/v0.1.1/guest/rust/examples/minigolf) by [SK83RJOSH](https://github.com/SK83RJOSH).
- Examples are now bundled into a downloadable `examples.zip` for each release.

### Fixed

- macOS ARM64 builds are now available after enabling the execution of unsigned executable memory (as required for wasmtime execution).
- The debugging configuration for VSCode was updated to use the new CLI.
- Minor documentation updates.

## Version 0.1.0 (2023-02-22)

Initial release. See the [announcement blog post](https://www.ambient.run/post/introducing-ambient).
