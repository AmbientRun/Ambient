# Chapter 4: Player interaction

It wouldn't be much of a game if we didn't have some player interaction though! Let's add that.

## A simple paint interaction

First we'll add a `Paint` message to our `ambient.toml`:

```toml
[message.Paint]
fields = { ray_origin = "Vec3", ray_dir = "Vec3" }
```

> Read more about defining your own messages [here](../../reference/messages.md).

Next, we'll add some code the `client.rs` (for the first time in this tutorial!):

```rust
Frame::subscribe(move |_| {
    let (input, _) = input::get_delta();
    if input.keys.contains(&KeyCode::Q) {
        let (camera, _) = query(active_camera()).build().evaluate().pop().unwrap();
        let ray = camera::screen_position_to_world_ray(camera, input.mouse_position);

        Paint {
            ray_origin: ray.origin,
            ray_dir: ray.dir,
        }
        .send_server_unreliable();
    }
});
```

> For an example of screen rays, [see this package](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/intermediate/screen_ray).

Then let's add this to our `server.rs`:

```rust
Paint::subscribe(|_, msg| {
    if let Some(hit) = physics::raycast_first(msg.ray_origin, msg.ray_dir) {
        Entity::new()
            .with(cube(), ())
            .with(translation(), hit.position)
            .with(scale(), Vec3::ONE * 0.1)
            .with(color(), vec4(0., 1., 0., 1.))
            .spawn();
    }
});
```

Great, let's run it and you should now be able to "paint" by pressing `Q`:


## [ ⇾ Chapter 5: Models](./5_models.md)
