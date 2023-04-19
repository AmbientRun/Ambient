# UI

Ambient's UI system is heavily inspired by React (with hooks), and follows many of the same patterns.
Take a look at the [React documentation](https://react.dev/reference/react) to learn how hooks work in general.

## Getting started

Here's a complete minimal example of a counter app:

```rust
use ambient_api::prelude::*;
use ambient_ui::prelude::*;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (count, set_count) = hooks.use_state(0);
    FlowColumn::el([
        Text::el(format!("We've counted to {count} now")),
        Button::new("Increase", move |_| set_count(count + 1)).el(),
    ])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
```

[See all UI examples here](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui).

## Layout

The layout is roughly based on [Windows Forms](https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout).

There are two major layout components, `Dock` and `Flow` (which includes `FlowColumn` and `FlowRow`).

`Dock` is top-down: it starts with a given area (say the screen) and then divides it into smaller pieces with each new element added to it.

`Flow` is bottom-up: it auto-resizes itself to fit its constituent components.
