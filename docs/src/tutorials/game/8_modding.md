# Chapter 8: Modding

In this final chapter, we'll look at modding. All games are moddable by default with Ambient.

## Adding the Mod Manager UI

We'll start by adding the Mod Manager UI package to your game, so that you can list and enable mods for your game. Start by adding the following to your `ambient.toml`'s dependencies:

```toml
package_manager = { deployment = "2nkcSe373rR3IdVwxkKHkj" }
```

(To find the latest deployment, visit the page for [the package mananger](https://ambient.run/packages/hr4pxz7kfhzgimicoyh65ydel3aehuhk).)

Then add this to the top of your `server.rs`'s main function:

```rust
entity::add_component(
    package_manager::entity(),
    package_manager::components::mod_manager_for(),
    packages::this::entity(),
);
```

Launch your game, and then press F4 to open the Mod Manager. From here, you can enable and disable mods. As your package is brand-new, there won't be any mods available yet. Let's fix that.

> **Note**: You can build your own Mod Manager UI if you want to. You can see the source code for the
> default one [here](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/packages/tools/package_manager).

## Creating a mod

To create a mod for your game, run `ambient new my_mod --rust empty`, then update the `ambient.toml` of the mod with this:

```toml
# Note: Add this field, or replace it if it already exists, making sure to update the ID:
content = { type = "Mod", for_playables = ["the_id_of_your_game_from_its_ambient_toml"] }

[dependencies]
# Note: This line will make it possible to run the mod locally, as it will pull in your game as a dependency.
# Replace LATEST_DEPLOYMENT_ID with the latest deployment ID of your game, which can be found on the game's page.
# When you want to deploy the mod, comment this line out first
my_game = { deployment = "LATEST_DEPLOYMENT_ID" }
```

You can now edit and run the code in `src/`, as per usual. Once you're happy with your mod, you can deploy it with `ambient deploy`, just like with the game. Providing a `screenshot.png` is recommended to make sure your mod stands out.

Remember to comment out the `my_game = ..` line before deploying.

This concludes the Ambient tutorial. Thanks for following along! If you have any questions, feel free to [join our Discord server](https://discord.gg/ambient) and ask away.

> **Source**: The complete code for this chapter can be found [here](https://github.com/AmbientRun/TutorialProject/tree/chapter-8).
