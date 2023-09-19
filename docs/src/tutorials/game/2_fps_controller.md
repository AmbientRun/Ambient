# Chapter 2: Fps controller

To get a bit of a head start, we're just going to use a ready made character and fps controller.
Start by adding the following to your `ambient.toml`:

```toml
[dependencies]
base_assets = { deployment = "5AHmgriArf3jPTcpSefBO" }
fps_controller = { deployment = "12Jw6s2ngUIpLA6pbiS3oJ" }
character_animation = { deployment = "42T888c8BcFMZqf80PhvYF" }
hide_cursor = { deployment = "6Vs97bmINdTpIoXuESfIcQ" }
```

> [Read more about dependencies here](../../reference/package.md#dependencies--dependencies)

Next, remove all code withing `fn main`, and add the following:

```rust
Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.0)
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(plane_collider(), ())
        .spawn();
```

This will create a basic ground plane for us. Save the file, and it should show some red squigly lines under the components; that's
because they aren't imported yet. Click one of them, then hit `ctrl-.` (or `cmd-.` on osx) and choose "Import ...".

> Entities are the basic unit in an ECS. You
> can think of the ECS as a database, where Entities are rows, and components (`quad`, `scale`, `color` and `plane_collider` in this case)
> are columns. Components are always pure data; they don't have any functionallity on their own. Instead, you typically
> write queries that read and write from the ECS (called "Systems").

> [Read more about the ECS here](../../reference/ecs.md)

Add the following code:

```rust
spawn_query(is_player()).bind(move |players| {
    for (id, _) in players {
        entity::add_components(
            id,
            Entity::new()
                .with(use_fps_controller(), ())
                .with(model_from_url(), base_assets::assets::url("Y Bot.fbx"))
                .with(basic_character_animations(), id),
        );
    }
});
```

> A `spawn_query` waits for an entity with a specific set of components to be spawned. In this case we're waiting
> for a player to be spawned. Then we're adding a few components to that player to give it an animated model, and
> to make it react to user input (the `use_fps_controller`). [Read more about queries here](../../reference/ecs.md#systems).

Run your game by pressing `F5` in VSCode (or by typing `ambient run` in your terminal).

You should now see something like this on the screen:

![Fps controller window](fps_controler.png)

Congratulations! You now have a basic character as a basis for the next of the tutorial.

