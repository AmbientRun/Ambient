# Distributing

This covers how to package and distribute games and assets.

## Deploying to the Ambient platform

Deploying to the [Ambient platform](https://ambient.run) is the easiest way to share
your content with the world. Any [package](./package.md) can be deployed with:

```sh
$ ambient deploy
Package "my-project" deployed successfully!
  Deployment ID: deployment_id
  Web URL: 'https://ambient.run/packages/package_id/deployment/deployment_id'
```

This will package and upload your creation to the platform. The web URL in the output can be used to browse your game or asset. If you're on a WebGPU enabled browser, the game can be played directly on the website. Any content upload to the Ambient platform is subject to our [terms of services](https://www.ambient.run/terms-of-service).

In the case your package is a game, it can be played directly on the website (if you're on a WebGPU enabled browser). Additionally, game servers will automatically be spun up when someone wants to play your game.

## Self-hosted

An important principle for us is "freedom of movement": if you don't want to use the Ambient platform for
your work, you don't have to. Packages can be deployed to your own filehost, and game servers can run
on your own platform.

As this path is less well-trodden, we're still working on making this as easy as possible. If you encounter any issues, please reach out to us on [Discord](https://discord.gg/ambient) or [GitHub](https://github.com/AmbientRun/Ambient/issues).

### Packages

To distribute your packages (games and assets) using your own servers, use `ambient build`, take the
resulting `build` folder and put it on any file server (e.g. a S3 bucket).

You can then use `ambient run https://address.to/your/content` to run that content.

### Game servers

We provide a [Docker image](https://github.com/AmbientRun/Ambient/pkgs/container/ambient) that can be used
to deploy your game servers.

## Distributing a desktop version of your game

It is possible to distribute a native desktop version of your game, but support for this is still experimental and subject to change. The assets will still be served from the Ambient platform/the URL you specify, but the game will run natively on the user's machine.

Create a `launch.json` that looks like this:

```json
{
  "args": ["run", "https://assets.ambient.run/1QI2Kc6xKnzantTL0bjiOQ"]
}
```

The address should point to a deployment of your game. `ambient deploy` can be used to deploy your game, and will give you an address back.

Package the `launch.json` together with the `ambient.exe` binary. The `ambient.exe` can be renamed to your liking (i.e. `my_game.exe`).

This can then be deployed to any platform that expects native desktop apps, including Steam and Epic Games.
