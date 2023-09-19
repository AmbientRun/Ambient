# Chapter 6: User interface (UI)

Many games rely on showing some kind of UI on top of the 3d game, so let's try adding some basic UI to our game.

## Showing the players position

Add the following to `client.rs`:

```rust
#[element_component]
fn PlayerPosition(hooks: &mut Hooks) -> Element {
    let (pos, _) = use_entity_component(hooks, player::get_local(), translation());
    Text::el(format!("Player position: {}", pos.unwrap_or_default()))
}
```

> In depth: UI in Ambient is losely inspired by React. See [the UI docs for more info](../../reference/ui.md).

> Tip: See [the UI examples](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui) to learn
> how to use layout, buttons, editors and much more.

And then add this to the `main` function in `client.rs`:

```rust
PlayerPosition.el().spawn_interactive();
```

You should now see something like this:

![UI](ui.png)

> Challenge: Try adding a `Button`, which when clicked sends a message to the server and moves the characters position somewhere else.

