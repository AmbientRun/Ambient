# Chapter 6: User interface (UI)

Many games rely on showing some kind of UI on top of the 3D game, so let's try adding some basic UI to our game.

## Showing the player's position

Switch to `client.rs`, and add the following:

```rust
#[element_component]
fn PlayerPosition(hooks: &mut Hooks) -> Element {
    let pos = use_entity_component(hooks, player::get_local(), translation());
    Text::el(format!("Player position: {}", pos.unwrap_or_default()))
}
```

> **In-depth**: UI in Ambient is loosely inspired by React. See [the UI reference documentation](../../reference/ui.md) for more information.

> **Tip**: See [the UI examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui) to learn how to use layout, buttons, editors and much more.

And then add this to the `main` function in `client.rs`:

```rust
PlayerPosition.el().spawn_interactive();
```

You should now see something like this:

![UI](ui.png)

> **Challenge**: Try adding a [`Button`](http://docs.ambient.run/nightly/ambient_api/prelude/struct.Button.html), which sends a message to the server and teleports the player somewhere else when clicked. You may find [Chapter 4](./4_player_interaction.md), the [button example](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/examples/ui/button/src/client.rs) and the [todo example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui/todo) useful.

## [⇾ Chapter 7: Deploying](./7_deploying.md)
