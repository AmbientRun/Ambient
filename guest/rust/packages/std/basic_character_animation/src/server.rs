use crate::packages::this::components::basic_character_animations;
use ambient_api::{
    animation::PlayClipFromUrlNodeRef,
    animation_element::{AnimationPlayer, BlendNode, PlayClipFromUrl, Transition},
    core::animation::components::{apply_animation_player, start_time},
    entity::get_component,
    prelude::*,
};
use packages::{
    this::assets,
    unit_schema::components::{health, jumping, run_direction, running},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

fn anim_url(name: &str) -> String {
    assets::url(&format!("{name}/animations/mixamo.com.anim"))
}

#[element_component(without_el)]
fn UnitAnimation(
    _hooks: &mut Hooks,
    direction: Vec2,
    running: bool,
    jumping: bool,
    health: f32,
) -> Element {
    AnimationPlayer {
        root: Transition {
            animations: vec![
                PlayClipFromUrl {
                    url: anim_url("Rifle Death.fbx"),
                    looping: false,
                }
                .el()
                .key("death"),
                PlayClipFromUrl {
                    url: anim_url("Rifle Jump.fbx"),
                    looping: false,
                }
                .el()
                .key("jump"),
                Walk { direction, running }.el(),
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
impl UnitAnimation {
    fn from_entity(entity: EntityId) -> Self {
        Self {
            direction: get_component(entity, run_direction()).unwrap_or_default(),
            running: get_component(entity, running()).unwrap_or_default(),
            jumping: get_component(entity, jumping()).unwrap_or_default(),
            health: get_component(entity, health()).unwrap_or(100.),
        }
    }
}

#[element_component]
fn Walk(hooks: &mut Hooks, running: bool, direction: Vec2) -> Element {
    let lagging_direction = hooks.use_ref_with(|_| Vec3::ZERO);
    let mut lag_dir = lagging_direction.lock();
    *lag_dir = lag_dir.lerp(direction.extend(if running { 1. } else { 0. }), 0.1);

    fn walkblend(items: &[(&str, f32)]) -> Element {
        BlendNode::normalize_multiblend(
            items
                .iter()
                .map(|(a, w)| {
                    (
                        PlayClipFromUrl {
                            url: anim_url(a),
                            looping: true,
                        }
                        .el()
                        .with(start_time(), Duration::ZERO),
                        1. - w.clamp(0., 1.),
                    )
                })
                .collect(),
        )
    }
    walkblend(&[
        ("Rifle Aiming Idle.fbx", lag_dir.xy().distance(Vec2::ZERO)),
        ("Walk Forward.fbx", lag_dir.distance(vec3(0., -1., 0.))),
        (
            "Walk Forward Left.fbx",
            lag_dir.distance(vec3(-1., -1., 0.)),
        ),
        (
            "Walk Forward Right.fbx",
            lag_dir.distance(vec3(1., -1., 0.)),
        ),
        ("Walk Backward.fbx", lag_dir.distance(vec3(0., 1., 0.))),
        (
            "Walk Backward Left.fbx",
            lag_dir.distance(vec3(-1., 1., 0.)),
        ),
        (
            "Walk Backward Right.fbx",
            lag_dir.distance(vec3(1., 1., 0.)),
        ),
        ("Walk Left.fbx", lag_dir.distance(vec3(-1., 0., 0.))),
        ("Walk Right.fbx", lag_dir.distance(vec3(1., 0., 0.))),
        ("Run Forward.fbx", lag_dir.distance(vec3(0., -1., 1.))),
        ("Run Forward Left.fbx", lag_dir.distance(vec3(-1., -1., 1.))),
        ("Run Forward Right.fbx", lag_dir.distance(vec3(1., -1., 1.))),
        ("Run Backward.fbx", lag_dir.distance(vec3(0., 1., 1.))),
        ("Run Backward Left.fbx", lag_dir.distance(vec3(-1., 1., 1.))),
        ("Run Backward Right.fbx", lag_dir.distance(vec3(1., 1., 1.))),
        ("Run Left.fbx", lag_dir.distance(vec3(-1., 0., 1.))),
        ("Run Right.fbx", lag_dir.distance(vec3(1., 0., 1.))),
    ])
}

#[main]
pub fn main() {
    fn preload(name: &str) {
        PlayClipFromUrlNodeRef::new(anim_url(name));
    }
    preload("Walk Forward.fbx");
    preload("Walk Backward.fbx");
    preload("Walk Left.fbx");
    preload("Walk Right.fbx");
    preload("Walk Forward Left.fbx");
    preload("Walk Forward Right.fbx");
    preload("Walk Backward Left.fbx");
    preload("Walk Backward Right.fbx");
    preload("Run Forward.fbx");
    preload("Run Backward.fbx");
    preload("Run Left.fbx");
    preload("Run Right.fbx");
    preload("Run Forward Left.fbx");
    preload("Run Forward Right.fbx");
    preload("Run Backward Left.fbx");
    preload("Run Backward Right.fbx");
    preload("Rifle Aiming Idle.fbx");

    let anims = Arc::new(Mutex::new(HashMap::<EntityId, ElementTree>::new()));

    spawn_query(basic_character_animations()).bind({
        let anims = anims.clone();
        move |v| {
            let mut anims = anims.lock().unwrap();
            for (id, target) in v {
                let tree = UnitAnimation::from_entity(id).el().spawn_tree();
                entity::add_component(
                    target,
                    apply_animation_player(),
                    tree.root_entity().unwrap(),
                );
                anims.insert(id, tree);
            }
        }
    });

    query(basic_character_animations()).each_frame(move |res| {
        let mut anims = anims.lock().unwrap();
        for (id, _) in res {
            let tree = anims.get_mut(&id).unwrap();
            tree.migrate_root(&mut World, UnitAnimation::from_entity(id).el());
            tree.update(&mut World);
        }
    });
}
