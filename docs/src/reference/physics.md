# Physics

Physics in Ambient is powered by Nvidia's PhysX ([user guide](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxguide/Manual/Index.html), [api doc](https://gameworksdocs.nvidia.com/PhysX/4.1/documentation/physxapi/files/index.html)).

## Colliders

To get started with physics, you'll need _colliders_, i.e. 3d shapes that represent objects. For example, to create a box, you'd do this:

```rust
Entity::new()
    .with(cube_collider(), Vec3::ONE)
    .spawn();
```

See the [api docs](https://docs.rs/ambient_api/0.2.1/ambient_api/components/core/physics/index.html) for other colliders.

The code above will create a physics collider, but you will also usually want to render something. To attach a visual cube to the
entity you'll simply do this:

```rust
Entity::new()
    .with(cube_collider(), Vec3::ONE)
    .with_merge(make_transformable())
    .with(cube(), ())
    .spawn();
```

## Dynamic objects

The code above will just create _static_ colliders, i.e. they won't move but are frozen in space. To create objects that move,
you'll need a collider together with the `physics_controlled` and `dynamic` components:

```rust
Entity::new()
    .with(cube_collider(), Vec3::ONE)
    .with_merge(make_transformable())
    .with(cube(), ())
    .with(physics_controlled(), ())
    .with(dynamic(), true)
    .spawn();
```

`physics_controlled` means that the entities transform in the ECS will by updated by the position from PhysX. `dynamic` means
it's an object that can move.

## Collision events

By using the `Collision` event you can get notifications when two objects collide with each other:

```rust
Collision::subscribe(move |msg| {
    println!("Bonk! {:?} collided", msg.ids);
});
```

## Colliders from models

You can also use model files as colliders (i.e. `.gltf` and `.fbx` files). Simply add this to your `pipeline.toml`:

```toml
[[pipelines]]
type = "Models"

[pipelines.collider]
type = "FromModel"
```

Which will create colliders for the models, and then spawn the object as follows:

```rust
Entity::new()
    .with_merge(make_transformable())
    .with(prefab_from_url(), assets::url("shape.glb"))
    .spawn();
```

## Examples

See [the physics example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/physics)
