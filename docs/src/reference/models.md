# Models

It's common to load models of things, such as characters, vehicles, buildings etc. into a game to have something
to render to the screen.

## Importing a model

To use a model in ambient, place it in the `assets` folder, and then create a `assets/pipeline.toml` file with the following content:

```toml
[[pipelines]]
type = "Models"
```

See [asset pipeline](./asset_pipeline.md) for more details.

## Spawning a model

You can then load and spawn your model like this:

```rust
Entity::new()
    .with_merge(make_transformable())
    .with(prefab_from_url(), asset::url("assets/MyModel.fbx").unwrap())
    .spawn();
```

A prefab is a visual model plus physics colliders. If the code above lives in your `server.rs` file, it will
create the physics colliders. The model (together with any skeletons it may have) will always be loaded and
spawned on the client side, regardless of if the above code lives in `server.rs` or `client.rs`.

You can also use `model_from_url` to only load the model, and not include the colliders.

## Animating a model

See [animations](./animations.md).

## Getting models for your project

See [getting content](./getting_content.md) for a list of places where you can get models.

## Manipulating bones

You can get individual bones of a loaded model using the `get_bone_by_bind_id` method.

```rust
let unit_id = Entity::new()
    .with_merge(make_transformable())
    .with(prefab_from_url(), asset::url("assets/MyModel.fbx").unwrap())
    .spawn();
let left_foot = get_bone_by_bind_id(unit_id, &BindId::LeftFoot).unwrap();
set_component(left_foot, rotation(), Quat::from_rotation_x(0.3));
```

This will only work on the client side as the skeleton is currently on loaded client-side.
