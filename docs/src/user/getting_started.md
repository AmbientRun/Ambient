# Getting started

Ambient projects are called _embers_. An ember is a collection of code, assets, and configuration that can be run in Ambient. They can be combined with other embers to create a game.

## Creating a ember

To create a ember, run:

```sh
ambient new my-ember
```

This will generate a new ember with a simple Rust module and an Ambient ember manifest.

## Running a ember

This ember can be run:

```sh
cd my-ember
ambient run
```

From here on, you can open up the ember in your favorite IDE and start editing the code. If you need a recommendation for an IDE, see [Setting up your IDE](./setting_up_ide.md). If using VS Code, the ember can be launched with the system-installed Ambient using the `F5`/Debug button, which is preconfigured to run the current ember.

For more details about the API, see [API](./api.md).

## Multiplayer

Every Ambient ember is multiplayer by default. To start the ember in server-only mode, use the following command:

```sh
ambient serve
```

This will output a line which looks like this:

```sh
[2023-04-13T09:05:42Z INFO  ambient_network::server] Proxy allocated an endpoint, use `ambient join proxy-eu.ambient.run:9898` to join
```

The server can now be connected to by anywhere on the internet (it's proxied by default), using the provided command:

```sh
ambient join proxy-eu.ambient.run:9898
```

Ambient always streams all assets, so the only thing anyone needs to connect to your server is Ambient itself. Try sending the command
to a friend, and play your game together!
