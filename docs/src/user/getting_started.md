# Getting started

## Creating a project

To create a project, run:

```sh
ambient new my-project
```

This will generate a new project with a simple Rust module and an Ambient project manifest.

## Running a project

This project can be run:

```sh
cd my-project
ambient run
```

From here on, you can open up the project in your favorite IDE and start editing the code. If you need a recommendation for an IDE, see [Setting up your IDE](./setting_up_ide.md). If you're using VSVode you can also launch the project inside of VSCode with F5.

For more details about the API, see [API](./api.md).

## Multiplayer

Every Ambient project is multiplayer by default. To start the project in server-only mode, use the following command:

```sh
ambient serve
```

This will output a line which looks like this:

```sh
[2023-04-13T09:05:42Z INFO  ambient_network::server] Proxy allocated an endpoint, use `ambient join proxy-eu.ambient.run:9898` to join
```

You can now connect to your server from anywhere on the internet (it's proxied by default), using the command it gave you:

```sh
ambient join proxy-eu.ambient.run:9898
```

Ambient always streams all assets, so the only thing anyone needs to connect to your server is Ambient itself. Try sending the command
to a friend, and play your game together!
