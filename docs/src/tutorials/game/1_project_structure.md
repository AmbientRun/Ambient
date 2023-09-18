# Chapter 1 Project structure

To start a new ambient project, type the following in your terminal of choice:

```sh
ambient new my_project
```

This will create a new project. Enter the project folder by typing `cd my_project`, and then try running it with:

```sh
ambient run
```

You should see a simple scene in front of you now, which looks like this:

![Ambient window](template.png)

Among the console output of ambient you should see a line that says something like:

```
Proxy allocated an endpoint, use `ambient join proxy-eu.ambient.run:9898` to join
```

This can be used to quickly test a multiplayer game; just copy the text in green and run it in another terminal window,
or even on another machine.

## Project structure

We write most of the game logic in `client.rs` and `server.rs`.

> Read more here about [where my code should go](https://ambientrun.github.io/Ambient/reference/faq.html#should-my-code-go-on-the-client-or-the-server)?

In `ambient.toml` we define some messages and components for the engine.

> You can read more about ECS [here](https://ambientrun.github.io/Ambient/reference/ecs.html).

In `Cargo.toml` we config Rust-related settings.

## Modify the code

Let's have a look at `client.rs` and `server.rs`.

`client.rs` is almost empty, but we will write things in it on later chapters.

In the `server.rs`, we created two entities: a camera and a plane(quad).

If you have installed all the recommended VS Code tools in the introduction page, you should be able to hover your mouse over each concept or component to see the docs:

![Code hint](hint.png)

## Challenge

Try to create some cubes and change their `translation()`, `scale()`, `rotation()` components.

> Hint: You need to have `.with_merge(make_transformable())` to be able to make those components effective.

> Tip: You can refer to the [primitives example](https://github.com/AmbientRun/Ambient/blob/main/guest/rust/examples/basics/primitives/src/server.rs) in the Ambient main GitHub repository.
