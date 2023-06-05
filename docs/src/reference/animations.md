# Animations

See the [skinmesh example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/skinmesh) for a complete example.

## Animation assets

To work with animations, you'll usually need some animation clips to work with. A good way to get started is
by going to [Mixamo](https://www.mixamo.com/#/) and download some characters and animation clips there.

Put your models and animations in your `assets` folder in your project, and make sure you have a `pipeline.json`
which can process models and animations, for instance something like this:

```json
{
    "pipeline": {
        "type": "Models"
    }
}
```

By running `ambient build` you can build the animations. You can browse the `build/assets` folder to see what
the build command produced.

## Animation player

To play animations, you'll need an `AnimationPlayer`. The animation player plays a graph of animations nodes;
currently the two nodes that exist are PlayClipFromUrlNode and BlendNode. Here's a basic example:

```rust
let clip = PlayClipFromUrlNode::new(
    asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
);
let player = AnimationPlayer::new(&clip);
```

To find out what the clip url is, you can look in your `build/assets/` folder.

You then need to also apply the animation to a model, for instance:
```rust
Entity::new()
    .with_merge(make_transformable())
    .with(
        prefab_from_url(),
        asset::url("assets/Peasant Man.fbx").unwrap(),
    )
    .with(apply_animation_player(), player.0)
    .spawn();
```

The same animation player can be attached to multiple models.

### Blending animations together

You can also blend two animations together:

```rust
let capoeira = PlayClipFromUrlNode::new(
    asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
);
let robot = PlayClipFromUrlNode::new(
    asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim").unwrap(),
);
let blend = BlendNode::new(&capoeira, &robot, 0.3);
let anim_player = AnimationPlayer::new(&blend);
```

This will blend capoeira (30%) and robot (70%) together, to form one output animation.

### Masked blending

You might want to mask blending as well, to for instance play one animation for the torso,
and one for the lower body:

```rust
let capoeira = PlayClipFromUrlNode::new(
    asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
);
let robot = PlayClipFromUrlNode::new(
    asset::url("assets/Robot Hip Hop Dance.fbx/animations/mixamo.com.anim").unwrap(),
);
let blend = BlendNode::new(&capoeira, &robot, 0.);
blend.set_mask_humanoid_lower_body(1.);
let anim_player = AnimationPlayer::new(&blend);
```

This will play the capoeira at the upper body, and the robot dance for the lower body.

### Attaching things to a skeleton

You can also attach things to a skeleton:

```rust
let left_foot = get_bone_by_bind_id(unit_id, &BindId::LeftFoot).unwrap();
let ball = Entity::new()
    .with_merge(make_transformable())
    .with_merge(make_sphere())
    .with(parent(), left_foot)
    .with_default(local_to_parent())
    // Without reset_scale, the ball would take the scale of the
    // bone we're attaching it to
    .with_default(reset_scale())
    .spawn();
add_child(left_foot, ball);
```

### Pre-loading animations

You can pre-load animations is by simply creating `PlayClipFromUrlNode` nodes and waiting for them to load:

```rust
let capoeira = PlayClipFromUrlNode::new(
    asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
);
capoeira.wait_until_loaded().await;
```

The clip will remain loaded as long as the object survives.

### Animation nodes lifetimes and ownership

The animation player and nodes all live in the ECS, and the `AnimationPlayer` and `PlayClipFromUrlNode` etc.
are all simple wrappers around an `EntityId`. To remove an animation player, call `player.despawn()` on it.

The animation nodes are ref counted, so the will survive as long as they are either being played by an
animation player, or as long as you have a reference to them in your code (i.e. you have an `PlayClipFromUrlNode`).
