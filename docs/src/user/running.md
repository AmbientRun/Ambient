# Running

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

From here on, you can open up the project in your favorite IDE and start editing the code. If you need a recommendation for an IDE, see [Setting up your IDE](./api.md#setting-up-your-ide).

For more details about the API, see [API](./api.md).

## Certificate

By default, _ambient_ bundles a self-signed certificate that is used by the server and trusted by the client.

To use your own certificate specify `--cert`, `--key`, for the server, and `--ca` for the client if the certificate authority which signed the certificate is not in the system roots. If specified, the bundled certificates will _not_ be used as a fallback.

```sh
ambient serve --cert ./localhost.crt --key ./localhost.key

```

```sh
ambient join 127.0.0.1:9000
```

**Note**: `--ca path_to_ca` must be specified if the used certificate is not in the system roots

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
