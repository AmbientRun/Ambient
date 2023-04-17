# Animations

There are multiple ways to work with animations, but the most common is to load animations from model files.

## Animations from model files

To get started with animations from model files (`.glb` and `.fbx` are supported), follow these steps;

1. Download an animation file and put it in a folder in the project, for instance `assets/Capoeira.fbx`.
   [Mixamo](https://www.mixamo.com/#/) is a great place to find animations for characters (as well as models for characters).
2. Add a `pipeline.json` in the `assets` folder, with the following content:
    ```json
    {
        "pipeline": {
            "type": "Models"
        }
    }
    ```
3. Run `ambient build`; this will process the animation file and put the output in the `build` folder.
4. Take a look in the `build` folder to find the name of the animations in the file; for instance `"assets/Capoeira.fbx/animations/mixamo.com.anim"`.
   `mixamo.com` is the name of the animation inside of the `Capoeira.fbx` file; you can also find that by opening the file in Blender. For fbx files
   the name may contain some extra metadata, so it may look something like `Armature|mixamo.com|Layer0`.
5. Load a character model and add the animation to it with the following code;
    ```rust
    let unit_id = Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            asset::url("assets/Peasant Man.fbx").unwrap(),
        )
        .spawn();

    entity::set_animation_controller(
        unit_id,
        AnimationController {
            actions: &[AnimationAction {
                clip_url: &asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap(),
                looping: true,
                weight: 1.,
            }],
            apply_base_pose: false,
        },
    );
    ```

See the [skinmesh example](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/basics/skinmesh) for a complete example.
