# Asset pipeline

<!-- markdownlint-disable-file MD024 -->

Ambient features an automated asset pipeline that is capable of loading and processing a number of assets and formats.

To use it, create a file named `pipeline.json` anywhere in your project. You can also prepend anything you'd like to the filename, which means `hello_pipeline.json` and such will also work. The full reference structure of the `json` is described in [Reference](#reference).

This pipeline will look at, but not necessarily process, all of the files adjacent to it in the folder. By convention, our examples place their assets in the `assets` folder, but this is not necessary.

A `pipeline.json` can contain one or more pipelines. To use more than one pipeline, wrap your pipeline object in a JSON array (`[]`).

## Models

The `Models` pipeline can be used to compile a model, or models, to meshes that can be used by Ambient. Additionally, by default, prefabs are created for each mesh. These prefabs can have components added to them automatically through the `object_components` field of the pipeline.

### Supported formats

- FBX: Native support
- glTF: Native support
- Unity models: Native support
- Quixel models: Native support
- ~30 other formats: This support is provided through the [assimp](https://github.com/assimp/assimp) library. It is not guaranteed to be fully integrated.

### Examples

#### Basic models

The following will load `.glb` and `.fbx` files in the folder or any of the sub-folders.

```json
{
  "pipeline": {
    "type": "Models"
  }
}
```

#### A more complex model example

The following will filter to just files that contain `table`, scale it down, and override materials for the `wood` material.

```json
{
  "pipeline": {
    "type": "Models",
    "collider": {
      "type": "FromModel"
    },
    "material_overrides": [
      {
        "filter": {
          "type": "ByName",
          "name": "wood"
        },
        "material": {
          "base_color": "wood_albedo.png",
          "metalic": 0.5,
          "roughness": 0.2
        }
      }
    ],
    "transforms": [
      {
        "type": "Scale",
        "scale": 0.1
      }
    ]
  },
  "sources": ["**/*table*"],
  "tags": ["Man made"]
}
```

### Notes

- If you are using components in your prefab and are hot-reloading it, the incoming prefab will overwrite any corresponding components on the current state of the entity. These components should only be used for static data - that is, `max_hitpoints` but not `current_hitpoints`.

## Materials

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

The full structure for `pipeline.json` is described below in TypeScript `.d.ts` format:

```typescript
{{#include pipeline.d.ts}}
```

In addition, a single `pipeline.json` can contain more than one pipeline. To do this, wrap your existing object in `[]` (i.e. a JSON array) and add more pipelines as required.
