# Project

All Ambient projects must have an `ambient.toml` project manifest that describes their functionality. This format is in flux, but is inspired by Rust's `Cargo.toml`.

At present, dependencies are _not_ supported, but this will change in future.

## WebAssembly

All `.wasm` components in the `build/{client, server}` directory will be loaded for the given target, regardless of provenance. The `.wasm` filenames must be snake-case ASCII identifiers, like the `id` in the manifest.

This means any `.wasm` that implements the Ambient [WIT interface](https://github.com/AmbientRun/Ambient/tree/main/crates/wasm/wit) and targets WASI snapshot 2 (or uses an adapter that targets WASI snapshot 2) should run within Ambient.

As a convenience for Rust users, Ambient will automatically build a `Cargo.toml` at the root of your project, if present, as `wasm32-wasi` for the features specified in `build.rust.feature-multibuild` in `ambient.toml` (defaults to `client` and `server`).

The default new project template will create `client.rs` and `server.rs` files, with a `Cargo.toml` preconfigured with targets for both. The resulting WASM bytecode files are then converted to components and placed in `build/{client, server}`.

The process it takes is equivalent to these commands:

```sh
cd your_project
cargo build --target wasm32-wasi --features client
wasm-tools component new target/wasm32-wasi/debug/your_project_client.wasm -o build/client/your_project.wasm --adapt wasi_snapshot_preview1.wasm
cargo build --target wasm32-wasi --features server
wasm-tools component new target/wasm32-wasi/debug/your_project_server.wasm -o build/server/your_project.wasm --adapt wasi_snapshot_preview1.wasm
```

using [wasm-tools](https://github.com/bytecodealliance/wasm-tools) and a bundled version of the [preview2-prototyping WASI adapter](https://github.com/bytecodealliance/preview2-prototyping).

## Reference

`Identifier`s are snake-case ASCII identifiers (as a string), and `IdentifierPath`s are a double-colon-separated list of `Identifier`s. For example, `my_project` is an `Identifier`, and `my_project::my_component` is an `IdentifierPath`.

### Project / `[project]`

The project section contains metadata about the project itself, such as its name and version.

| Property      | Type         | Description                                                                                   |
| ------------- | ------------ | --------------------------------------------------------------------------------------------- |
| `id`          | `Identifier` | _Required_. The project's snake-cased ID.                                                     |
| `name`        | `String`     | _Required_. A human-readable name for the project.                                            |
| `description` | `String`     | _Required_. A human-readable description of the project.                                      |
| `version`     | `String`     | _Required_. The project's version, in `(major, minor, patch)` format. Semantically versioned. |

### Build / `[build]`

The build section contains settings related to building the project.

#### Rust Settings / `[build.rust]`

| Property             | Type       | Description                                                                                                                                                                                                                                                                                                                  |
| -------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `feature-multibuild` | `String[]` | _Optional_. An array of strings defining the features to be used when building the project. This is used to build the same code for both client and server.<br /><br />Client and server are built by default (e.g. `["client", "server"]`); this is exposed so that you can disable building one side entirely if required. |

### Components / `[components]`

The components section contains custom components defined by the project. Components are used to store data on entities.

This is a TOML table, where the keys are the component IDs (`IdentifierPath`), and the values are the component definitions.

| Property      | Type                   | Description                                                |
| ------------- | ---------------------- | ---------------------------------------------------------- |
| `type`        | `ComponentType`        | _Required_. The type of the component.                     |
| `name`        | `String`               | _Required_. A human-readable name for the component.       |
| `description` | `String`               | _Required_. A human-readable description of the component. |
| `attributes`  | `ComponentAttribute[]` | _Optional_. An array of attributes for the component.      |

A `ComponentType` is either:

- a string that can be one of the following primitive types:

  - `Bool`
  - `Empty`
  - `EntityId`
  - `F32`
  - `F64`
  - `I32`
  - `Mat4`
  - `Quat`
  - `String`
  - `U32`
  - `U64`
  - `U8`
  - `Uvec2`
  - `Uvec3`
  - `Uvec4`
  - `Vec2`
  - `Vec3`
  - `Vec4`

- a contained type of the form `{ type = "Vec", element_type = ComponentType }` or `{ type = "Option", element_type = ComponentType }`
  - Note that `Vec` and `Option` are the only supported container types, and `element_type` must be a primitive `ComponentType` (that is, you cannot have nested contained types).

A `ComponentAttribute` is a string that can be one of the following:

- `Debuggable`: this component can have its debug value printed, especially in ECS dumps
- `Networked`: this component is networked
- `Resource`: this component will only ever be used as a resource; will error if attached to an entity
- `MaybeResource`: this component can be used as a resource or as a component; necessary if treating this component as a resource
- `Store`: this component's value should be persisted when the world is saved

### Concepts / `[concepts]`

The concepts section contains custom concepts defined by the project. Concepts are used to define a set of components that can be attached to an entity.

This is a TOML table, where the keys are the concept IDs (`Identifier`), and the values are the concept definitions.

| Property      | Type                       | Description                                                                                                                    |
| ------------- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `name`        | `String`                   | _Required_. A human-readable name for the concept.                                                                             |
| `description` | `String`                   | _Required_. A human-readable description of the concept.                                                                       |
| `extends`     | `String[]`                 | _Optional_. An array of concepts to extend. Must be defined in this project manifest.                                          |
| `components`  | `Map<IdentifierPath, any>` | _Required_. An object containing the components and their default values. Must be components defined in this project manifest. |

The `components` is an object where the keys are `IdentifierPath`s of components defined in the project manifest (at this time, it must be in the same manifest), and the values are the default values for those components in the concept.

### Messages / `[messages]`

The messages section contains custom messages defined by the project. Messages are used to communicate between client and server.

For an example of how to use messages, see the [messaging example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/messaging).

This is a TOML table, where the keys are the message IDs (`Identifier`), and the values are the message definitions.

| Property      | Type                             | Description                                                                                                     |
| ------------- | -------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `name`        | `String`                         | _Required_. A human-readable name for the message.                                                              |
| `description` | `String`                         | _Required_. A human-readable description of the message.                                                        |
| `fields`      | `Map<Identifier, ComponentType>` | _Required_. An object containing the fields and their types. Must be one of the types supported for components. |

## Sample `ambient.toml`

A sample `ambient.toml` is shown below:

<!-- TODO: autogenerate with generate-docs -->

```toml
{{#include ambient.sample.toml}}
```
