# Chapter 5: Working with models

To make games a bit more interesting we usually work with 3d models (rather than just cubes and spheres like we've been working with so far).

Let's download [this free sample model from the offical gltf sample repository](https://github.com/KhronosGroup/glTF-Sample-Models/blob/master/2.0/AntiqueCamera/glTF-Binary/AntiqueCamera.glb).
Click the little download icon to the right to download it.

Next, create a folder named `assets` in your project, and add the file to that folder (see [package structure](./1_package.md#package-structure)).

Then create a file called `pipeline.toml` in the `assets` folder, with the following content:

```toml
[[pipelines]]
type = "Models"
sources = ["*.glb"]
```

> [Read more about asset pipelines here](../../reference/asset_pipeline.md)

Finally, let's use the model. In our `server.rs`, add the following lines:

```rust
Entity::new()
    .with_merge(Transformable {
        local_to_world: Default::default(),
        optional: TransformableOptional {
            scale: Some(Vec3::ONE * 0.3),
            ..Default::default()
        },
    })
    .with(model_from_url(), assets::url("AntiqueCamera.glb"))
    .spawn();
```

You should now see something like this:

![Model](model.png)

Great! We've learned how to load models into Ambient.

> Tip: Use `prefab_from_url` instead of `model_from_url` if you also want to include a collider. [See the physics example.](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/physics)
