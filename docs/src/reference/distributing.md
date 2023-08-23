# Distributing your game

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
