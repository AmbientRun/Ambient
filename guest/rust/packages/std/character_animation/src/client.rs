use crate::packages::this::components::basic_character_animations;
use ambient_api::{
    animation::PlayClipFromUrlNodeRef,
    animation_element::{AnimationPlayer, BlendNode, PlayClipFromUrl, Transition},
    core::animation::components::{apply_animation_player, start_time},
    element::use_ref_with,
    entity::get_component,
    prelude::*,
};
use packages::{
    game_object::components::health,
    this::assets,
    this::components::*,
    unit_schema::components::{jumping, run_direction, running},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

fn anim_url(name: &str) -> String {
    assets::url(&format!("{name}.fbx/animations/mixamo.com.anim"))
}

fn clip(owner: EntityId, component: Component<String>, fallback: &str) -> String {
    if let Some(url) = get_component(owner, component) {
        url
    } else {
        anim_url(fallback)
    }
}

#[element_component(without_el)]
fn UnitAnimation(
    _hooks: &mut Hooks,
    owner_id: EntityId,
    direction: Vec2,
    running: bool,
    jumping: bool,
    health: f32,
) -> Element {
    AnimationPlayer {
        root: Transition {
            animations: vec![
                PlayClipFromUrl {
                    url: clip(owner_id, death(), "Rifle Death"),
                    looping: false,
                }
                .el()
                .key("death"),
                PlayClipFromUrl {
                    url: clip(owner_id, jump(), "Rifle Jump"),
                    looping: false,
                }
                .el()
                .key("jump"),
                Walk {
                    owner_id,
                    direction,
                    running,
                }
                .el(),
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
            owner_id: entity,
            direction: get_component(entity, run_direction()).unwrap_or_default(),
            running: get_component(entity, running()).unwrap_or_default(),
            jumping: get_component(entity, jumping()).unwrap_or_default(),
            health: get_component(entity, health()).unwrap_or(100.),
        }
    }
}

#[element_component]
fn Walk(hooks: &mut Hooks, owner_id: EntityId, running: bool, direction: Vec2) -> Element {
    let lagging_direction = use_ref_with(hooks, |_| Vec3::ZERO);
    let mut lag_dir = lagging_direction.lock();
    *lag_dir = lag_dir.lerp(direction.extend(if running { 1. } else { 0. }), 0.1);

    fn walkblend(items: &[(String, f32)]) -> Element {
        BlendNode::normalize_multiblend(
            items
                .iter()
                .map(|(a, w)| {
                    (
                        PlayClipFromUrl {
                            url: a.clone(),
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
    let f = Vec3::X;
    let r = Vec3::Y;
    let run = Vec3::Z;
    walkblend(&[
        (
            clip(owner_id, idle(), "Rifle Aiming Idle"),
            lag_dir.xy().distance(Vec2::ZERO),
        ),
        (
            clip(owner_id, walk_forward(), "Walk Forward"),
            lag_dir.distance(f),
        ),
        (
            clip(owner_id, walk_forward_left(), "Walk Forward Left"),
            lag_dir.distance(f - r),
        ),
        (
            clip(owner_id, walk_forward_right(), "Walk Forward Right"),
            lag_dir.distance(f + r),
        ),
        (
            clip(owner_id, walk_backward(), "Walk Backward"),
            lag_dir.distance(-f),
        ),
        (
            clip(owner_id, walk_backward_left(), "Walk Backward Left"),
            lag_dir.distance(-f - r),
        ),
        (
            clip(owner_id, walk_backward_right(), "Walk Backward Right"),
            lag_dir.distance(-f + r),
        ),
        (
            clip(owner_id, walk_left(), "Walk Left"),
            lag_dir.distance(-r),
        ),
        (
            clip(owner_id, walk_right(), "Walk Right"),
            lag_dir.distance(r),
        ),
        (
            clip(owner_id, run_forward(), "Run Forward"),
            lag_dir.distance(f + run),
        ),
        (
            clip(owner_id, run_forward_left(), "Run Forward Left"),
            lag_dir.distance(f - r + run),
        ),
        (
            clip(owner_id, run_forward_right(), "Run Forward Right"),
            lag_dir.distance(f + r + run),
        ),
        (
            clip(owner_id, run_backward(), "Run Backward"),
            lag_dir.distance(-f + run),
        ),
        (
            clip(owner_id, run_backward_left(), "Run Backward Left"),
            lag_dir.distance(-f - r + run),
        ),
        (
            clip(owner_id, run_backward_right(), "Run Backward Right"),
            lag_dir.distance(-f + r + run),
        ),
        (
            clip(owner_id, run_left(), "Run Left"),
            lag_dir.distance(-r + run),
        ),
        (
            clip(owner_id, run_right(), "Run Right"),
            lag_dir.distance(r + run),
        ),
    ])
}

#[main]
pub fn main() {
    fn preload(name: &str) {
        PlayClipFromUrlNodeRef::new(anim_url(name));
    }
    preload("Walk Forward");
    preload("Walk Backward");
    preload("Walk Left");
    preload("Walk Right");
    preload("Walk Forward Left");
    preload("Walk Forward Right");
    preload("Walk Backward Left");
    preload("Walk Backward Right");
    preload("Run Forward");
    preload("Run Backward");
    preload("Run Left");
    preload("Run Right");
    preload("Run Forward Left");
    preload("Run Forward Right");
    preload("Run Backward Left");
    preload("Run Backward Right");
    preload("Rifle Aiming Idle");

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
