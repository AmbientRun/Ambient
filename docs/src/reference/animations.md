# Animations

See the [skinmesh example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/skinmesh) for a full runable example.

## Animation assets

To work with animations, you'll need some animation clips to work with. A good way to get started is
by going to [Mixamo](https://www.mixamo.com/#/) and download some characters and animations there.

Put your models and animations in your `assets` folder in your project, and make sure you have a `pipeline.json`
which can process models and animations, for instance something like this:

```toml
[[pipelines]]
type = "Models"
```

### Finding the clip urls

By running `ambient build` you build the assets. You can browse the `build/assets` folder to see what
the build command produced.

If we look at the [skinmesh example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/skinmesh), you can see that we have an animation called `assets/Capoeira.fbx`. If you build this project,
you'll see that it produces a file called `build/assets/Capoeira.fbx/animations/mixamo.com.anim`. If you
remove the `build/` part of that, you will have the animation clip url; i.e. `assets/Capoeira.fbx/animations/mixamo.com.anim`.

## Animation player

To play animations, you'll need an `AnimationPlayer`. The animation player plays a graph of animations nodes;
currently the two nodes that exist are `PlayClipFromUrlNode` and `BlendNode`. Here's an example of how to set
it up to play a single animation:

```rust
let clip = PlayClipFromUrlNode::new(
    asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
);
let player = AnimationPlayer::new(&clip);

// Let's load a character model to apply the animation to.
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

To blend two animations together you can use the `BlendNode`:

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

It's quite common two want to play one animation for the torso and another for the lower
body of a character model. To achieve this we use masking:

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

The value you pass in is the masked blend weight. In the previous example, we saw that a blend
weight of 0.3 would play 30% of the capoeira animation. The same concept can be applied _per-bone_
instead. So for instance, setting `BlendNode::new(&capoeira, &robot, 0.3)` and then
`blend.set_mask_humanoid_lower_body(0.9)` means that all bones play capoeira 30%, except for the
lower body which plays it 90%.

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

This will spawn a ball and attach it to the left foot of the character.

### Pre-loading animations

You can pre-load animations is by simply creating `PlayClipFromUrlNode` nodes and waiting for them to load:

```rust
let capoeira = PlayClipFromUrlNode::new(
    asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
);
capoeira.wait_until_loaded().await;
```

The clip will remain loaded as long as the object survives.

### Retargeting

It's possible to play an animation that was made for one character on another character. To make
it look right, you might need to use retargeting though. Try setting retargeting on the `PlayClipFromUrlNode` (`set_retargeting`), and/or use `apply_base_pose` on the same node.

If you're using mixamo for animations, you can do retargeting through mixamo itself to get the best results.

### Animation nodes lifetimes and ownership

The animation player and nodes all live in the ECS, and the `AnimationPlayer` and `PlayClipFromUrlNode` etc.
are all simple wrappers around an `EntityId`. To remove an animation player, call `player.despawn()` on it.

The animation nodes are ref counted, so the will survive as long as they are either being played by an
animation player, or as long as you have a reference to them in your code (i.e. you have an `PlayClipFromUrlNode`).
