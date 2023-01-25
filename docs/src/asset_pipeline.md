# Asset pipeline

Tilt supports processing and loading a number of assets. You can place a file that ends with `pipeline.json` anywhere in your
project folder which will specify how the assets will be processed. You can also prepend anything you'd like to the filename,
so for instance `hello_pipeline.json` will also work.

For a full reference, see [pipelines/mod.rs](https://github.com/TiltOrg/Tilt/blob/main/crates/build/src/pipelines/mod.rs#L31). Supported
model formats are:

- FBX and GLTF are natively supported
- ~30 other formats are supported through assimp, though less well supported.
- Unity and Quixel can be loaded

Here are some examples:

## Basic models

The following will load `.glb` and `.fbx` files in the folder or any of the sub-folders.

```json
[
    {
        "pipeline": {
            "type": "Models"
        }
    }
]
```

## A more complex model example

The following will filter to just files that contain `table`, scale it down, and override materials for the `wood` material.

```json
[
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
        "tags": [
            "Man made"
        ]
    }
]
```
