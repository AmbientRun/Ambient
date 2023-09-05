use ambient_api::{
    animation::PlayClipFromUrlNodeRef,
    animation_element::{AnimationPlayer, BlendNode, PlayClipFromUrl, Transition},
    core::{
        animation::components::{apply_animation_player, start_time},
        player::components::is_player,
    },
    prelude::*,
};
use packages::{
    afps_schema::components::{
        player_direction, player_health, player_jumping, player_model_ref, player_running,
    },
    this::assets,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

fn anim_url(name: &str) -> String {
    assets::url(&format!("{name}/animations/mixamo.com.anim"))
}

#[element_component]
fn UnitAnimation(
    _hooks: &mut Hooks,
    direction: Vec2,
    running: bool,
    jumping: bool,
    health: i32,
) -> Element {
    println!(
        "UnitAnimation running: {running}, jumping: {jumping}, health: {health} direction: {:?}",
        direction
    );
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
            active: if health <= 0 {
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

    spawn_query((is_player(), player_model_ref())).bind({
        let anims = anims.clone();
        move |v| {
            let mut anims = anims.lock().unwrap();
            for (id, (_, model)) in v {
                let tree = UnitAnimation {
                    direction: Vec2::ZERO,
                    running: false,
                    jumping: false,
                    health: 100,
                }
                .el()
                .spawn_tree();
                entity::add_component(model, apply_animation_player(), tree.root_entity().unwrap());
                anims.insert(id, tree);

                entity::add_component(id, player_jumping(), false);
            }
        }
    });

    query((
        is_player(),
        player_model_ref(),
        player_direction(),
        player_running(),
        player_health(),
        player_jumping(),
    ))
    .each_frame(move |res| {
        let mut anims = anims.lock().unwrap();
        for (player_id, (_, _model, dir, is_running, health, jump)) in res {
            let tree = anims.get_mut(&player_id).unwrap();
            tree.migrate_root(
                &mut World,
                UnitAnimation {
                    direction: dir,
                    running: is_running,
                    jumping: jump,
                    health,
                }
                .el(),
            );
            tree.update(&mut World);
        }
    });
}
