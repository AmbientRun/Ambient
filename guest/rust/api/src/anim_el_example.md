# Animation Element module

The animation element module provides a simple animation system you can use as in the follwoing example

```no_run
#[element_component(without_el)]
fn CharacterAnimation(
    _hooks: &mut Hooks,
    animations: Animations,
    direction: Vec2,
    running: bool,
    jumping: bool,
    health: f32,
) -> Element {
    AnimationPlayer {
        root: Transition {
            animations: vec![
                PlayClipFromUrl {
                    url: animations.death.clone(),
                    looping: false,
                }
                .el()
                .key("death"),
                PlayClipFromUrl {
                    url: animations.jump.clone(),
                    looping: false,
                }
                .el()
                .key("jump"),
                PlayClipFromUrl {
                    url: animations.idle.clone(),
                    looping: true,
                }
                .el()
                .key("idle"),
            ],
            active: if health <= 0. {
                0
            } else if jumping {
                1
            } else {
                2
            },
            speed: 0.3,
        }
        .el(),
    }
    .el()
}

impl CharacterAnimation {
    fn from_entity(entity: EntityId) -> Self {
        Self {
            direction: entity::get_component(entity, run_direction()).unwrap_or_default(),
            running: entity::get_component(entity, running()).unwrap_or_default(),
            jumping: entity::get_component(entity, jumping()).unwrap_or_default(),
            health: entity::get_component(entity, health()).unwrap_or(100.),
        }
    }
}
```

Later, attach the animation tree to an entity

```no_run
let anims = Arc::new(Mutex::new(HashMap::<EntityId, ElementTree>::new()));
spawn_query(<your component>).bind({
    let anims = anims.clone();
    move |v| {
        let mut anims = anims.lock().unwrap();
        for (id, target) in v {
            let target = if target.is_null() { id } else { target };
            let tree = CharacterAnimation::from_entity(id).el().spawn_tree();
            entity::add_component(
                target,
                apply_animation_player(),
                tree.root_entity().unwrap(),
            );
            anims.insert(id, tree);
        }
    }
});
```

On your frame loop, update the animation tree

```no_run
    query(,your component).each_frame(move |res| {
        let mut anims = anims.lock().unwrap();
        for (id, _) in res {
            let tree = anims.get_mut(&id).unwrap();
            tree.migrate_root(&mut World, CharacterAnimation::from_entity(id).el());
            tree.update(&mut World);
        }
    });
```
