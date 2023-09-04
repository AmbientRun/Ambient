# Animations

See the [skinmesh example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/skinmesh) for a complete example.

## Animation assets

To work with animations, you will need some animation clips. A good way to get started is
by going to [Mixamo](https://www.mixamo.com/#/) and downloading some characters and animations.

In the `assets` folder of your package, place your models and animations. Additionally, in the same folder,
make sure you have a `pipeline.toml` which can process models and animations:

```toml
[[pipelines]]
type = "Models"
```

### Finding the clip URLs

The `ambient build` command will build the assets. You can browse the `build/assets` folder to see what
was produced by the command.

As an example:

- The [skinmesh example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/skinmesh)
  has an animation called `assets/Capoeira.fbx`.
- The build process will produce `build/ambient_example_skinmesh/assets/Capoeira.fbx/animations/mixamo.com.anim`.
- The animation clip URL is this path after `assets/`: `Capoeira.fbx/animations/mixamo.com.anim`.

In the following examples, it is assumed that you have imported `assets` from a package, like so:

```rust
use packages::ambient_example_skinmesh::assets;
```

## Animation player

An `AnimationPlayerRef` is used to play animations. The player executes a graph of animation nodes; at present,
the two nodes that exist are `PlayClipFromUrlNodeRef` and `BlendNodeRef`.

Here's an example of how to set up a graph and play it for a single animation:

```rust
let clip = PlayClipFromUrlNodeRef::new(
    assets::url("Capoeira.fbx/animations/mixamo.com.anim")
);
let player = AnimationPlayerRef::new(&clip);

// Let's load a character model to apply the animation to.
Entity::new()
    .with_merge(make_transformable())
    .with(prefab_from_url(), assets::url("Peasant Man.fbx"))
    .with(apply_animation_player(), player.0)
    .spawn();
```

The same animation player can be attached to multiple models.

### Blending animations together

A `BlendNodeRef` can be used to blend two animations together:

```rust
let capoeira = PlayClipFromUrlNodeRef::new(
    assets::url("Capoeira.fbx/animations/mixamo.com.anim")
);
let robot = PlayClipFromUrlNodeRef::new(
    assets::url("Robot Hip Hop Dance.fbx/animations/mixamo.com.anim")
);
let blend = BlendNodeRef::new(&capoeira, &robot, 0.3);
let anim_player = AnimationPlayerRef::new(&blend);
```

This will blend `capoeira` (30%) and `robot` (70%) together to form one output animation.

### Masked blending

A common use case for blending is to blend two animations together for different parts of the body;
this is achieved using masking. Here's an example of how to blend two animations together for the upper and lower
body:

```rust
let capoeira = PlayClipFromUrlNodeRef::new(
    assets::url("Capoeira.fbx/animations/mixamo.com.anim")
);
let robot = PlayClipFromUrlNodeRef::new(
    assets::url("Robot Hip Hop Dance.fbx/animations/mixamo.com.anim")
);

let blend = BlendNodeRef::new(&capoeira, &robot, 0.0);
blend.set_mask_humanoid_lower_body(1.0);

let anim_player = AnimationPlayerRef::new(&blend);
```

This will play the `capoeira` at the upper body, and the `robot` dance for the lower body.
The `set_mask_humanoid_lower_body` and `set_mask_humanoid_upper_body` functions are convenience
functions for setting the mask for the upper and lower body.

The blend node's weight is still relevant when used with masking, but can also be set per-bone using the mask.
Setting `BlendNodeRef::new(&capoeira, &robot, 0.3)` and then `blend.set_mask_humanoid_lower_body(0.9)` will play all
nodes in the `capoeira` animation at 30%, except for the lower body, which will play it at 90%. If no mask is set,
the weight is used for all bones.

### Attaching entities to a skeleton

Entities can be attached to bones on a skeleton. This is done by adding a `parent` component to the entity that
points to the bone to be attached to. The entity should also have a `local_to_parent` component, which will be
the transformation of the entity relative to the bone. For more information, see the documentation on [hierarchies](hierarchies.md).

```rust
let left_foot = animation::get_bone_by_bind_id(unit_id, &BindId::LeftFoot).unwrap();
let ball = Entity::new()
    .with_merge(make_transformable())
    .with_merge(make_sphere())
    .with(parent(), left_foot)
    .with(local_to_parent(), Default::default())
    // Without reset_scale, the ball would take the scale of the
    // bone we're attaching it to
    .with(reset_scale(), ())
    .spawn();
entity::add_child(left_foot, ball);
```

This will spawn a ball and attach it to the left foot of the character.

### Pre-loading animations

Animations can be pre-loaded by creating a `PlayClipFromUrlNodeRef` node and waiting for it to load:

```rust
let capoeira = PlayClipFromUrlNodeRef::new(
    assets::url("Capoeira.fbx/animations/mixamo.com.anim")
);
capoeira.wait_for_load().await;
```

The clip will remain loaded as long as the object survives.

### Retargeting

It is possible to play an animation that was made for one character on another character.
Retargeting may be necessary to remap the animation from the original character's skeleton to your target
character's skeleton.

To do this, `PlayClipFromUrlNodeRef::set_retargeting` can be used to configure the retargeting for a given clip.
Additionally, `PlayClipFromUrlNodeRef::apply_base_pose` may be necessary to change the origin of the animation
for correctness.

If you're using Mixamo for animations, you can do retargeting through Mixamo itself to get the best results.

### Animation nodes lifetimes and ownership

The animation player and nodes all live in the ECS. The `AnimationPlayerRef`, `PlayClipFromUrlNodeRef` and other nodes
are wrappers around an `EntityId`. You are responsible for despawning them when you're done with them, by calling
`.despawn()`, which will remove the node and all the children.
