# Ember

All Ambient embers must have an `ambient.toml` manifest that describes their functionality. This format is in flux, but is inspired by Rust's `Cargo.toml`.

## WebAssembly

All `.wasm` components in the `build/{client, server}` directory will be loaded for the given target, regardless of provenance. The `.wasm` filenames must be snake-case ASCII identifiers, like the `id` in the manifest.

This means any `.wasm` that implements the Ambient [WIT interface](https://github.com/AmbientRun/Ambient/tree/main/crates/wasm/wit) and targets WASI snapshot 2 (or uses an adapter that targets WASI snapshot 2) should run within Ambient.

As a convenience for Rust users, Ambient will automatically build a `Cargo.toml` at the root of your ember, if present, as `wasm32-wasi` for the features specified in `build.rust.feature-multibuild` in `ambient.toml` (defaults to `client` and `server`).

The default new ember template will create `client.rs` and `server.rs` files, with a `Cargo.toml` preconfigured with targets for both. The resulting WASM bytecode files are then converted to components and placed in `build/{client, server}`.

The process it takes is equivalent to these commands:

```sh
cd your_ember
cargo build --target wasm32-wasi --features client
wasm-tools component new target/wasm32-wasi/debug/your_ember_client.wasm -o build/client/your_ember.wasm --adapt wasi_snapshot_preview1.wasm
cargo build --target wasm32-wasi --features server
wasm-tools component new target/wasm32-wasi/debug/your_ember_server.wasm -o build/server/your_ember.wasm --adapt wasi_snapshot_preview1.wasm
```

using [wasm-tools](https://github.com/bytecodealliance/wasm-tools) and a bundled version of the [preview2-prototyping WASI adapter](https://github.com/bytecodealliance/preview2-prototyping).

## Reference

- `SnakeCaseIdentifier`s are snake-case ASCII identifiers (as a string)
- `PascalCaseIdentifier`s are PascalCase ASCII identifiers (as a string)
- `Identifiers` are either a `SnakeCaseIdentifier` or a `PascalCaseIdentifier` based on context
- `ItemPath`s are a double-colon-separated list of `SnakeCaseIdentifier`s followed by a single `Identifier`. For example, `my_ember` is an `Identifier`, and `my_ember::my_component` is an `ItemPath`.

### `ValueType`

In Ambient, all typed values must have a type that belongs to `ValueType`. This includes component types and message fields.

A `ValueType` is either:

- a string that can be one of the following primitive types:

  - `Bool`: a boolean value, true or false
  - `Empty`: a component that has no value; most often used for tagging an entity
  - `EntityId`: an entity ID
  - `F32`: a 32-bit floating point value
  - `F64`: a 64-bit floating point value
  - `Mat4`: a 4x4 32-bit floating point matrix
  - `Quat`: a 32-bit floating point quaternion
  - `String`: a UTF-8 string
  - `U8`: an 8-bit unsigned integer value
  - `U16`: an 16-bit unsigned integer value
  - `U32`: a 32-bit unsigned integer value
  - `U64`: a 64-bit unsigned integer value
  - `I8`: an 8-bit signed integer value
  - `I16`: an 16-bit signed integer value
  - `I32`: a 32-bit signed integer value
  - `I64`: a 64-bit signed integer value
  - `Uvec2`: a 2-element 32-bit unsigned integer vector
  - `Uvec3`: a 3-element 32-bit unsigned integer vector
  - `Uvec4`: a 4-element 32-bit unsigned integer vector
  - `Ivec2`: a 2-element 32-bit signed integer vector
  - `Ivec3`: a 3-element 32-bit signed integer vector
  - `Ivec4`: a 4-element 32-bit signed integer vector
  - `Vec2`: a 2-element 32-bit floating point vector
  - `Vec3`: a 3-element 32-bit floating point vector
  - `Vec4`: a 4-element 32-bit floating point vector
  - `Duration`: A time span. Often used as a timestamp, in which case it designates the duration since Jan 1, 1970.

- a contained type of the form `{ type = "Vec", element_type = ValueType }` or `{ type = "Option", element_type = ValueType }`

  - Note that `Vec` and `Option` are the only supported container types, and `element_type` must be a primitive `ValueType` (that is, you cannot have nested contained types).

- a string that refers to an `enum` defined by an ember; see [Enums](#enums--enums).

Note that `ValueType`s are not themselves values, but rather types of values. For example, `Vec2` is a `ValueType`, but `Vec2(1.0, 2.0)` is a value of type `Vec2`. Additionally, `ValueType`s from other embers can be referred to using `ItemPath`s: `my_ember::my_component::MyType`.

### Ember / `[ember]`

The `ember` section contains metadata about the ember itself, such as its name and version.

| Property      | Type                  | Description                                                                                 |
| ------------- | --------------------- | ------------------------------------------------------------------------------------------- |
| `id`          | `SnakeCaseIdentifier` | _Required_. The ember's snake-cased ID.                                                     |
| `name`        | `String`              | _Optional_. A human-readable name for the ember.                                            |
| `description` | `String`              | _Optional_. A human-readable description of the ember.                                      |
| `version`     | `String`              | _Optional_. The ember's version, in `(major, minor, patch)` format. Semantically versioned. |

#### Example

```toml
#
# The ember section describes all ember metadata.
#
[ember]
# This must be a snake-cased name.
id = "my_cool_ember"
# This name is human-readable and can contain anything. Optional.
name = "My Cool Ember"
# This description is human-readable and can contain anything. Optional.
description = "A sample ember that's the coolest thing ever."
# Embers are expected to use (major, minor, patch) semantic versioning.
# Other formats are not accepted. This requirement may be relaxed later.
# Optional, but required for deployments.
version = "0.0.1"
```

### Build / `[build]`

The build section contains settings related to building the ember.

#### Rust Settings / `[build.rust]`

| Property             | Type       | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| -------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `feature-multibuild` | `String[]` | _Optional_. An array of strings defining the Rust features to be used when building the ember. This is used to build the same code for both client and server.<br /><br />`cargo build` will be run with each of these features to produce a separate WASM binary, which is then componentized and copied into a folder of the corresponding name in `build/`.<br /><br />Client and server are built by default (e.g. `["client", "server"]`); this is exposed so that you can disable building one side entirely if required. |

#### Example

```toml
[build.rust]
feature-multibuild = ["client", "server"]
```

### Components / `[components]`

The `components` section contains custom components defined by the ember. Components are used to store data on entities.

This is a TOML table, where the keys are the component IDs (`SnakeCaseIdentifier`), and the values are the component definitions.

| Property      | Type                   | Description                                                |
| ------------- | ---------------------- | ---------------------------------------------------------- |
| `type`        | `ValueType`            | _Required_. The type of the component.                     |
| `name`        | `String`               | _Optional_. A human-readable name for the component.       |
| `description` | `String`               | _Optional_. A human-readable description of the component. |
| `attributes`  | `ComponentAttribute[]` | _Optional_. An array of attributes for the component.      |

A `ComponentAttribute` is a string that can be one of the following:

- `Debuggable`: this component can have its debug value printed, especially in ECS dumps
- `Networked`: this component is networked
- `Resource`: this component will only ever be used as a resource; will error if attached to an entity
- `MaybeResource`: this component can be used as a resource or as a component; necessary if treating this component as a resource
- `Store`: this component's value should be persisted when the world is saved

#### Example

```toml
[components]
# Inline tables can be used.
cool_component = { type = "I32", name = "Cool Component", description = "A cool component", attributes = ["Debuggable"] }

# Explicit tables can also be used.
[components.cool_component2]
type = "I32"
name = "Cool Component 2"
description = "A cool component 2"
attributes = ["Debuggable"]
```

### Concepts / `[concepts]`

The `concepts` section contains custom concepts defined by the ember. Concepts are used to define a set of components that can be attached to an entity.

This is a TOML table, where the keys are the concept IDs (`SnakeCaseIdentifier`), and the values are the concept definitions.

| Property      | Type                 | Description                                                                         |
| ------------- | -------------------- | ----------------------------------------------------------------------------------- |
| `name`        | `String`             | _Optional_. A human-readable name for the concept.                                  |
| `description` | `String`             | _Optional_. A human-readable description of the concept.                            |
| `extends`     | `String[]`           | _Optional_. An array of concepts to extend. Must be defined in this ember manifest. |
| `components`  | `Map<ItemPath, any>` | _Required_. An object containing the components and their default values.           |

The `components` is an object where the keys are `ItemPath`s of components defined in the ember manifest, and the values are the default values for those components in the concept.

#### Example

```toml
[concepts.concept1]
name = "Concept 1"
description = "The best"
[concepts.Concept1.components]
cool_component = 0

# A concept that extends `concept1` and has both `cool_component` and `cool_component2`.
[concepts.concept2]
extends = ["Concept1"]
components = { cool_component2 = 1 }
```

### Messages / `[messages]`

The `messages` section contains custom messages defined by the ember. Messages are used to communicate between client and server.

For an example of how to use messages, see the [messaging example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/intermediate/messaging).

This is a TOML table, where the keys are the message IDs (`PascalCaseIdentifier`), and the values are the message definitions.

| Property      | Type                                  | Description                                                                                                     |
| ------------- | ------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `description` | `String`                              | _Optional_. A human-readable description of the message.                                                        |
| `fields`      | `Map<SnakeCaseIdentifier, ValueType>` | _Required_. An object containing the fields and their types. Must be one of the types supported for components. |

#### Example

```toml
[messages.Input]
description = "Describes the input state of the player."
[messages.Input.fields]
# Each field in the message must have a type.
direction = "Vec2"
mouse_delta_x = "F32"
```

### Enums / `[enums]`

The `enums` section contains custom enums defined by the ember. Enums are used to define a closed set of values.

This is a TOML table, where the keys are the ember IDs (`PascalCaseIdentifier`), and the values are the ember definitions.

| Property      | Type                                | Description                                                                                        |
| ------------- | ----------------------------------- | -------------------------------------------------------------------------------------------------- |
| `description` | `String`                            | _Optional_. A human-readable description of the enum.                                              |
| `members`     | `Map<PascalCaseIdentifier, String>` | _Required_. An object containing the members and their descriptions. The description can be empty. |

#### Example

```toml
[enums.CakeBakeState]
description = "Describes the state of a cake bake."
[enums.CakeBakeState.members]
GatheringIngredients = "Gathering ingredients"
MixingIngredients = "Mixing ingredients"
Baking = "Baking"
Cooling = "Cooling"
Decorating = "Decorating"
Done = "Done"
```

### Dependencies / `[dependencies]`

The `dependencies` section contains a list of ember IDs that this ember depends on.

Depending on another ember gives you access to its items, including its components, concepts, messages, and enums. It can also provide access to any assets that the ember has.

This is a TOML table, where the keys are the name that you want to access this ember by (`SnakeCaseIdentifier`), and the location of the ember is the value.

To access an item from an ember, use the following syntax: `import_name::item_id`. For example, if you have an ember imported with the name `the_basics` and an enum with ID `BasicEnum`, you can access it with `the_basics::BasicEnum`.

At the time of writing, only path dependencies are supported. This is likely to change in future.

| Property  | Type     | Description                                                                                                        |
| --------- | -------- | ------------------------------------------------------------------------------------------------------------------ |
| `path`    | `String` | _Required_. A relative path to the ember to depend on.                                                             |
| `enabled` | `bool`   | _Optional_. Control whether or not logic associated with this ember should be enabled on load. Enabled by default. |

For an example of how to use dependencies, see the [dependencies example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/intermediate/dependencies).

#### Example

```toml
[dependencies]
the_basics = { path = "../basics" }

[components]
my_component = { type = "the_basics::BasicEnum" }
```
