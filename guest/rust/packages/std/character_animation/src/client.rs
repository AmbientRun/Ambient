use ambient_api::{
    animation::PlayClipFromUrlNodeRef,
    animation_element::{AnimationPlayer, BlendNode, PlayClipFromUrl, Transition},
    core::animation::components::{apply_animation_player, start_time},
    element::use_ref_with,
    prelude::*,
};
use packages::{
    game_object::components::health,
    this::{
        assets,
        components::{basic_character_animations, *},
    },
    unit_schema::components::{jumping, run_direction, running},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

#[element_component(without_el)]
fn UnitAnimation(
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
                Walk {
                    animations: animations.clone(),
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
        let use_rifle_animations =
            entity::has_component(entity, packages::this::components::use_rifle_animations());

        Self {
            animations: Animations::for_entity(
                entity,
                if use_rifle_animations {
                    Animations::rifle()
                } else {
                    Animations::standard()
                },
            ),
            direction: entity::get_component(entity, run_direction()).unwrap_or_default(),
            running: entity::get_component(entity, running()).unwrap_or_default(),
            jumping: entity::get_component(entity, jumping()).unwrap_or_default(),
            health: entity::get_component(entity, health()).unwrap_or(100.),
        }
    }
}

#[element_component]
fn Walk(hooks: &mut Hooks, animations: Animations, running: bool, direction: Vec2) -> Element {
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
    let ld = *lag_dir;

    walkblend(&[
        (animations.idle, ld.xy().distance(Vec2::ZERO)),
        (animations.walk_forward, ld.distance(f)),
        (animations.walk_forward_left, ld.distance(f - r)),
        (animations.walk_forward_right, ld.distance(f + r)),
        (animations.walk_backward, ld.distance(-f)),
        (animations.walk_backward_left, ld.distance(-f - r)),
        (animations.walk_backward_right, ld.distance(-f + r)),
        (animations.walk_left, ld.distance(-r)),
        (animations.walk_right, ld.distance(r)),
        (animations.run_forward, ld.distance(f + run)),
        (animations.run_forward_left, ld.distance(f - r + run)),
        (animations.run_forward_right, ld.distance(f + r + run)),
        (animations.run_backward, ld.distance(-f + run)),
        (animations.run_backward_left, ld.distance(-f - r + run)),
        (animations.run_backward_right, ld.distance(-f + r + run)),
        (animations.run_left, ld.distance(-r + run)),
        (animations.run_right, ld.distance(r + run)),
    ])
}

#[main]
pub fn main() {
    // Preload all animation sets
    for set in [Animations::standard(), Animations::rifle()] {
        for anim in set.all() {
            PlayClipFromUrlNodeRef::new(anim);
        }
    }

    let anims = Arc::new(Mutex::new(HashMap::<EntityId, ElementTree>::new()));

    spawn_query(basic_character_animations()).bind({
        let anims = anims.clone();
        move |v| {
            let mut anims = anims.lock().unwrap();
            for (id, target) in v {
                let target = if target.is_null() { id } else { target };
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

// TODO: See if this could potentially be done using concepts
// and storing the standard/rifle animations in entities
#[derive(Debug, Clone)]
struct Animations {
    walk_forward: String,
    walk_backward: String,
    walk_left: String,
    walk_right: String,

    walk_forward_left: String,
    walk_forward_right: String,
    walk_backward_left: String,
    walk_backward_right: String,

    run_forward: String,
    run_backward: String,
    run_left: String,
    run_right: String,
    run_forward_left: String,
    run_forward_right: String,
    run_backward_left: String,
    run_backward_right: String,

    idle: String,
    death: String,
    jump: String,
}
impl Animations {
    fn all(&self) -> [&str; 19] {
        [
            &self.walk_forward,
            &self.walk_backward,
            &self.walk_left,
            &self.walk_right,
            &self.walk_forward_left,
            &self.walk_forward_right,
            &self.walk_backward_left,
            &self.walk_backward_right,
            &self.run_forward,
            &self.run_backward,
            &self.run_left,
            &self.run_right,
            &self.run_forward_left,
            &self.run_forward_right,
            &self.run_backward_left,
            &self.run_backward_right,
            &self.idle,
            &self.death,
            &self.jump,
        ]
    }

    fn standard() -> &'static Animations {
        static STANDARD: OnceLock<Animations> = OnceLock::new();
        STANDARD.get_or_init(|| Animations {
            walk_forward: raw_anim_url("standard", "Walking"),
            walk_backward: raw_anim_url("standard", "Walking_Backward"),
            walk_left: raw_anim_url("standard", "Left_Strafe_Walk"),
            walk_right: raw_anim_url("standard", "Right_Strafe_Walking"),

            walk_forward_left: raw_anim_url("standard", "Left_Strafe_Walk"),
            walk_forward_right: raw_anim_url("standard", "Right_Strafe_Walking"),
            walk_backward_left: raw_anim_url("standard", "Left_Strafe_Walk"),
            walk_backward_right: raw_anim_url("standard", "Right_Strafe_Walking"),

            run_forward: raw_anim_url("standard", "Running_1"),
            run_backward: raw_anim_url("standard", "Running_Backward"),
            run_left: raw_anim_url("standard", "Left_Strafe"),
            run_right: raw_anim_url("standard", "Right_Strafe"),

            run_forward_left: raw_anim_url("standard", "Jog_Forward_Diagonal_left"),
            run_forward_right: raw_anim_url("standard", "Jog_Forward_Diagonal_right"),
            run_backward_left: raw_anim_url("standard", "Jog_Backward_Diagonal_left"),
            run_backward_right: raw_anim_url("standard", "Jog_Backward_Diagonal_right"),

            idle: raw_anim_url("standard", "Idle"),
            death: raw_anim_url("rifle", "Rifle Death"),
            jump: raw_anim_url("rifle", "Rifle Jump"),
        })
    }

    fn rifle() -> &'static Animations {
        static RIFLE: OnceLock<Animations> = OnceLock::new();
        RIFLE.get_or_init(|| Animations {
            walk_forward: raw_anim_url("rifle", "Walk Forward"),
            walk_backward: raw_anim_url("rifle", "Walk Backward"),
            walk_left: raw_anim_url("rifle", "Walk Left"),
            walk_right: raw_anim_url("rifle", "Walk Right"),

            walk_forward_left: raw_anim_url("rifle", "Walk Forward Left"),
            walk_forward_right: raw_anim_url("rifle", "Walk Forward Right"),
            walk_backward_left: raw_anim_url("rifle", "Walk Backward Left"),
            walk_backward_right: raw_anim_url("rifle", "Walk Backward Right"),

            run_forward: raw_anim_url("rifle", "Run Forward"),
            run_backward: raw_anim_url("rifle", "Run Backward"),
            run_left: raw_anim_url("rifle", "Run Left"),
            run_right: raw_anim_url("rifle", "Run Right"),

            run_forward_left: raw_anim_url("rifle", "Run Forward Left"),
            run_forward_right: raw_anim_url("rifle", "Run Forward Right"),
            run_backward_left: raw_anim_url("rifle", "Run Backward Left"),
            run_backward_right: raw_anim_url("rifle", "Run Backward Right"),

            idle: raw_anim_url("rifle", "Rifle Aiming Idle"),
            death: raw_anim_url("rifle", "Rifle Death"),
            jump: raw_anim_url("rifle", "Rifle Jump"),
        })
    }

    fn for_entity(id: EntityId, fallback: &Animations) -> Self {
        let get = |component: Component<String>, getter: fn(&Animations) -> &String| {
            entity::get_component(id, component).unwrap_or_else(|| getter(fallback).to_string())
        };

        Animations {
            walk_forward: get(walk_forward(), |f| &f.walk_forward),
            walk_backward: get(walk_backward(), |f| &f.walk_backward),
            walk_left: get(walk_left(), |f| &f.walk_left),
            walk_right: get(walk_right(), |f| &f.walk_right),

            walk_forward_left: get(walk_forward_left(), |f| &f.walk_forward_left),
            walk_forward_right: get(walk_forward_right(), |f| &f.walk_forward_right),
            walk_backward_left: get(walk_backward_left(), |f| &f.walk_backward_left),
            walk_backward_right: get(walk_backward_right(), |f| &f.walk_backward_right),

            run_forward: get(run_forward(), |f| &f.run_forward),
            run_backward: get(run_backward(), |f| &f.run_backward),
            run_left: get(run_left(), |f| &f.run_left),
            run_right: get(run_right(), |f| &f.run_right),

            run_forward_left: get(run_forward_left(), |f| &f.run_forward_left),
            run_forward_right: get(run_forward_right(), |f| &f.run_forward_right),
            run_backward_left: get(run_backward_left(), |f| &f.run_backward_left),
            run_backward_right: get(run_backward_right(), |f| &f.run_backward_right),

            idle: get(idle(), |f| &f.idle),
            death: get(death(), |f| &f.death),
            jump: get(jump(), |f| &f.jump),
        }
    }
}

fn raw_anim_url(prefix: &str, name: &str) -> String {
    assets::url(&format!("{prefix}/{name}.fbx/animations/mixamo.com.anim"))
}
