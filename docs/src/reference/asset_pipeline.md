# Asset pipeline

<!-- markdownlint-disable-file MD024 -->

Ambient features an automated asset pipeline that is capable of loading and processing a number of assets and formats.

To use it, create a file ending in `...pipeline.toml`, such as `pipeline.toml` but also `hello_pipeline.toml`.

This pipelines will look at, but not necessarily process, all of the files adjacent to it in the folder. By convention,
our examples place their assets in the `assets` folder, but this is not necessary.

## Models

The `Models` pipeline can be used to compile a model, or models, to meshes that can be used by Ambient. Additionally, by
default, prefabs are created for each mesh. These prefabs can have components added to them automatically through the
`object_components` field of the pipeline.

### Supported formats

- FBX: Native support
- glTF: Native support
- Unity models: Native support
- Quixel models: Native support
- ~30 other formats: This support is provided through the [assimp](https://github.com/assimp/assimp) library. It is not
guaranteed to be fully integrated.

### Examples

#### Basic models

The following will load `.glb` and `.fbx` files in the folder or any of the sub-folders.

```toml
[[pipelines]]
type = "Models"
```

#### Different pipelines for different files

You can use the `sources` attribute to restrict different configurations to different files:

```toml
[[pipelines]]
type = "Models"
sources = [ "physical/*.glb" ]

[pipelines.collider]
type = "FromModel"

[[pipelines]]
type = "Models"
sources = [ "ghosts/*.glb" ]
```

`sources` accepts a list of glob patterns, so you can target a single file or a pattern to select all files in a
directory (`*.glb`) or sub-tree (`**/test.glb`).

#### A more complex model example

The following will filter to just files that contain `table`, scale it down, and override materials for the `wood`
material.

```toml
{{ #include ../../../guest/rust/examples/basics/asset_loading/assets/pipeline.toml }}
```

### Notes

- If you are using components in your prefab and are hot-reloading it, the incoming prefab will overwrite any
corresponding components on the current state of the entity. These components should only be used for static data - that
is, `max_hitpoints` but not `current_hitpoints`.

## Models
### Regular

Consumes model file formats into a hierarchy of entities, materials, and meshes.

#### Supported formats:
- `glb`
- `gltf`
- `fbx`
- `obj`

### Unity

Consumes Unity packages processing all meshes, textures and materials, and *lod* levels into a normalized form to consume in Ambient.
Usage of a processed model during runtime is identical to `Regular`.

### Quixel

Import Quixel packages

Supports collections, lod-levels, etc.

## Materials

Import materials from a collecting of maps such as *base_color*, *normals maps*.

Detailed documentation is pending, but please consult the [Reference](#reference).

### Supported formats

- `jpg`
- `png`
- `gif`
- `webp`
- as well as other common image formats

## Audio

Detailed documentation is pending, but please consult the [Reference](#reference).

### Supported formats

- `ogg`
- `wav`
- `mp3`

## Reference

See `rustdoc` for a complete reference of supported pipelines, model importers, material configurations,
and the like.

```sh
cargo doc --open -p ambient_pipeline_types
```
