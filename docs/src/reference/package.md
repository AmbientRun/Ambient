# Package

All Ambient packages must have an `ambient.toml` manifest that describes their functionality. This format is in flux, but is inspired by Rust's `Cargo.toml`.

## WebAssembly

All `.wasm` components in the `build/{client, server}` directory will be loaded for the given target, regardless of provenance. The `.wasm` filenames must be snake-case ASCII identifiers, like the `id` in the manifest.

This means any `.wasm` that implements the Ambient [WIT interface](https://github.com/AmbientRun/Ambient/tree/main/crates/wasm/wit) and targets WASI snapshot 2 (or uses an adapter that targets WASI snapshot 2) should run within Ambient.

As a convenience for Rust users, Ambient will automatically build a `Cargo.toml` at the root of your package, if present, as `wasm32-wasi` for the features specified in `build.rust.feature-multibuild` in `ambient.toml` (defaults to `client` and `server`).

The default new package template will create `client.rs` and `server.rs` files, with a `Cargo.toml` preconfigured with targets for both. The resulting WASM bytecode files are then converted to components and placed in `build/{client, server}`.

The process it takes is equivalent to these commands:

```sh
cd your_package
cargo build --target wasm32-wasi --features client
wasm-tools component new target/wasm32-wasi/debug/your_package_client.wasm -o build/client/your_package.wasm --adapt wasi_snapshot_preview1.wasm
cargo build --target wasm32-wasi --features server
wasm-tools component new target/wasm32-wasi/debug/your_package_server.wasm -o build/server/your_package.wasm --adapt wasi_snapshot_preview1.wasm
```

using [wasm-tools](https://github.com/bytecodealliance/wasm-tools) and a bundled version of the [preview2-prototyping WASI adapter](https://github.com/bytecodealliance/preview2-prototyping).

## Rust

Rust is a first-class language for Ambient packages. The default new package template will create `client.rs` and `server.rs` files, with a `Cargo.toml` preconfigured with targets for both.

The API provides a `#[main]` attribute macro that generates code to allow you to access the data and functionality of the packages known to your package. All packages, including your own, will be in the `packages` module.

## Reference

- `SnakeCaseIdentifier`s are snake-case ASCII identifiers (as a string)
- `PascalCaseIdentifier`s are PascalCase ASCII identifiers (as a string)
- `Identifiers` are either a `SnakeCaseIdentifier` or a `PascalCaseIdentifier` based on context
- `ItemPath`s are a double-colon-separated list of `SnakeCaseIdentifier`s followed by a single `Identifier`. For example, `my_package` is an `Identifier`, and `my_package::my_component` is an `ItemPath`.

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

- a string that refers to an `enum` defined by a package; see [Enums](#enums--enums).

Note that `ValueType`s are not themselves values, but rather types of values. For example, `Vec2` is a `ValueType`, but `Vec2(1.0, 2.0)` is a value of type `Vec2`. Additionally, `ValueType`s from other packages can be referred to using `ItemPath`s: `my_package::my_component::MyType`.

### Package / `[package]`

The `package` section contains metadata about the package itself, such as its name and version.

| Property      | Type                  | Description                                                                                       |
| ------------- | --------------------- | ------------------------------------------------------------------------------------------------- |
| `id`          | `SnakeCaseIdentifier` | _Required_. The package's snake-cased ID.                                                         |
| `name`        | `String`              | _Optional_. A human-readable name for the package.                                                |
| `description` | `String`              | _Optional_. A human-readable description of the package.                                          |
| `version`     | `String`              | _Optional_. The package's version, in `(major, minor, patch)` format. Semantically versioned.     |
| `content`     | `PackageContent`      | _Required_. A description of the content of this Package. See below.                              |
| `public`      | `Bool`                | _Optional_. Indicates if this package will be publicly available when deployed. Defaults to true. |

#### `PackageContent`

These are the valid configurations for package content:

```toml
# A Playable is anything that can be run as an application; i.e. games, examples, applications etc.
content = { type = "Playable" }
content = { type = "Playable", example = true } # example defaults to false

# Assets are things you can use as a dependency in your package
content = { type = "Asset", models = true, textures = true } # Contains models and textures
# These are the valid asset types:
#
#   models
#   animations
#   textures
#   materials
#   audio
#   fonts
#   code
#   schema
#
# You can use any combination of them

# Tools are things you can use to develop your package
content = { type = "Tool" }

# Mods are extension to Playables
content = { type = "Mod", for_playables: ["i3terk32jw"] }
```

#### Example

```toml
#
# The package section describes all package metadata.
#
[package]
# This must be a snake-cased name.
id = "my_cool_package"
# This name is human-readable and can contain anything. Optional.
name = "My Cool Package"
# This description is human-readable and can contain anything. Optional.
description = "A sample package that's the coolest thing ever."
# Packages are expected to use (major, minor, patch) semantic versioning.
# Other formats are not accepted. This requirement may be relaxed later.
# Optional, but required for deployments.
version = "0.0.1"
content = { type = "Asset", code = true }
```

### Build / `[build]`

The build section contains settings related to building the package.

#### Rust Settings / `[build.rust]`

| Property             | Type       | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| -------------------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `feature-multibuild` | `String[]` | _Optional_. An array of strings defining the Rust features to be used when building the package. This is used to build the same code for both client and server.<br /><br />`cargo build` will be run with each of these features to produce a separate WASM binary, which is then componentized and copied into a folder of the corresponding name in `build/`.<br /><br />Client and server are built by default (e.g. `["client", "server"]`); this is exposed so that you can disable building one side entirely if required. |

#### Example

```toml
[build.rust]
feature-multibuild = ["client", "server"]
```

### Components / `[components]`

The `components` section contains custom components defined by the package. Components are used to store data on entities.

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

The `concepts` section contains custom concepts defined by the package. Concepts are used to define a set of components that can be attached to an entity.

This is a TOML table, where the keys are the concept IDs (`SnakeCaseIdentifier`), and the values are the concept definitions.

| Property      | Type                 | Description                                                                                                                                                                                                                                                                                                                                                                           |
| ------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`        | `String`             | _Optional_. A human-readable name for the concept.                                                                                                                                                                                                                                                                                                                                    |
| `description` | `String`             | _Optional_. A human-readable description of the concept.                                                                                                                                                                                                                                                                                                                              |
| `extends`     | `String[]`           | _Optional_. An array of concepts to extend. Must be defined in this package manifest.                                                                                                                                                                                                                                                                                                 |
| `components`  | `Map<ItemPath, any>` | _Required_. An object containing the components and their default values.<br /><br />`Mat4` and `Quat` support `Identity` as a string, which will use the relevant identity value for that type.<br /><br />`F32` and `F64` support `PI`, `FRAC_PI_2`, `-PI`, and `-FRAC_PI_2` as string values, which correspond to pi (~3.14), half-pi (~1.57), and negative versions respectively. |

The `components` is an object where the keys are `ItemPath`s of components defined in the package manifest, and the values are the default values for those components in the concept.

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

The `messages` section contains custom messages defined by the package. Messages are used to communicate between client and server, or between packages/modules on the same side.

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

The `enums` section contains custom enums defined by the package. Enums are used to define a closed set of values.

This is a TOML table, where the keys are the package IDs (`PascalCaseIdentifier`), and the values are the package definitions.

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

### Includes / `[includes]`

The `includes` section contains a list of manifests to pull in under a given name. This is useful for splitting up a package into multiple files.

This is a TOML table, where the keys are the name that you want to access this include by (`SnakeCaseIdentifier`), and the location of the package manifest is the value.

#### Example

```toml
[includes]
graphics = "graphics/ambient.toml"
```

### Dependencies / `[dependencies]`

The `dependencies` section contains a list of package IDs that this package depends on.

Depending on another package gives you access to its items, including its components, concepts, messages, and enums. It can also provide access to any assets that the package has.

This is a TOML table, where the keys are the name that you want to access this package by (`SnakeCaseIdentifier`), and the location of the package is the value.

To access an item from a package, use the following syntax: `import_name::item_id`. For example, if you have a package imported with the name `the_basics` and an enum with ID `BasicEnum`, you can access it with `the_basics::BasicEnum`.

At least one of `path`, `url` or `deployment` must be specified.

| Property     | Type     | Description                                                                                                          |
| ------------ | -------- | -------------------------------------------------------------------------------------------------------------------- |
| `path`       | `String` | A relative path to the package to depend on.                                                                         |
| `url`        | `Url`    | A URL to a deployed package.                                                                                         |
| `deployment` | `String` | The ID of a deployed package to depend on.                                                                           |
| `enabled`    | `bool`   | _Optional_. Control whether or not logic associated with this package should be enabled on load. Enabled by default. |

For an example of how to use dependencies, see the [dependencies example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/intermediate/dependencies).

#### Example

```toml
[dependencies]
the_basics = { path = "../basics" }

[components]
my_component = { type = "the_basics::BasicEnum" }
```

### Runtime access to packages

Packages are represented as entities within the ECS, with their metadata being stored as components. This means that you can access the metadata of a package at runtime. To do so, you can use the `entity()` function inside the generated Rust code for the package:

```rust
use ambient_api::prelude::*;

#[main]
fn main() {
    dbg!(entity::get_all_components(packages::this::entity()));
}
```

Or by querying for entities that have the `is_package` component:

```rust
use ambient_api::{
    core::package::components::{is_package, name},
    prelude::*,
};

#[main]
fn main() {
    let q = query((is_package(), name())).build();
    // List all packages and their names.
    dbg!(q.evaluate());
}
```
