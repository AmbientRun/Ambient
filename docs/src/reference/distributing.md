# Distributing

This covers how to package and distribute games and assets.

## Deploying to the Ambient cloud

Deploying to the official Ambient cloud (https://ambient.run) is the absolutely easiest way. Any package,
which can be a game, asset, tool or mod, can be simply be deployed with:

```bash
$ ambient deploy
Deploying... (231mb)
Done! Your package can be found at https://ambient.run/packages/QejgH0BvdSUcWnR2Q45Ug
```

This will package and upload your creation to the Ambient cloud. In the case of games, this will also
automatically create game servers for you when someone wants to play your game. The command will return
an url which you can use to browse your game or asset (and if you're on a WebGPU enabled browser you can
even play it right there on the website). Any content upload to the Ambient cloud is subject to our
[terms of services](https://www.ambient.run/terms-of-service).

## Self hosted

And important principle for us is "Freedom of movement"; if you don't want to use the Ambient cloud for
your work then you don't have to. Packages can be deployed to your own file host, and game servers can run
in your own cloud.

### Packages

To distribute your packages (games and assets) to your own servers, you simply run `ambient build`, take the
resulting `build` folder and put it on any file server (s3 bucket for instance). You can then run
`ambient run https://address.to/your/content` to run that content.

### Game servers

We're providing a [docker image](https://github.com/AmbientRun/Ambient/pkgs/container/ambient) that can be used
to deploy your game servers.

## Distributing a desktop version of your game

It's possible to distribute a native desktop version of your game by following these steps:

First, create a `launch.json`, like this:

```json
{
    "args": ["run", "https://assets.ambient.run/1QI2Kc6xKnzantTL0bjiOQ"]
}
```

The address should point to a deployment of your game. `ambient deploy` can be used to deploy your game, and will give you an address back.

Then package the `launch.json` together with the `ambient.exe` binary. The `ambient.exe` can be renamed to your liking (i.e. `my_game.exe`).
This can then be deployed to any platform that expects native desktop apps, such as Steam and Epic games.
