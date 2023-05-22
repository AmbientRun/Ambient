use ambient_api::{
    components::core::{
        camera::aspect_ratio_from_window, prefab::prefab_from_url, primitives::quad,
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    entity::{AnimationAction, AnimationController},
    prelude::*,
};

use crate::components::{
    bone_tracker_binders, bone_tracker_enabled, bone_tracker_entities, bone_tracker_index,
    bone_tracker_markers,
};

#[main]
pub async fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), vec3(2., 2., 3.0))
        .with(lookat_target(), vec3(0., 0., 1.))
        .spawn();

    Entity::new()
        .with_merge(make_transformable())
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with(name(), "Floor".to_string())
        .spawn();

    let unit_id = Entity::new()
        .with_merge(make_transformable())
        .with(
            prefab_from_url(),
            asset::url("assets/Peasant Man.fbx").unwrap(),
        )
        .with(name(), "Peasant".to_string())
        .spawn();

    let clip_url = &asset::url("assets/Capoeira.fbx/animations/mixamo.com.anim").unwrap();
    entity::set_animation_controller(
        unit_id,
        AnimationController {
            actions: &[
                AnimationAction {
                    clip_url,
                    looping: true,
                    weight: 1.,
                },
            ],
            apply_base_pose: false,
        },
    );


    // NOTE: Work around since the animations components are not exposed in ECS yet
    sleep(2.0).await;


    BoneTracker::add_system();
    BoneTracker::setup(unit_id);
    BoneTracker::el(unit_id).spawn_interactive();
}

#[element_component]
fn BoneSelect(
    hooks: &mut Hooks,
    name: String,
    unit_id: EntityId,
    index: usize,
) -> Element {
    let (selected, set_selected) = hooks.use_state(false);

    BoneTracker::set_enabled_by_index(unit_id, index, selected);

    Button::new_inner(
        if !selected {
            Text::el(name).small_style()
        } else {
            Text::el(name).header_style()
        },
        cb(move |_world| {
            set_selected(!selected);
        }),
    )
    .el()
}

#[element_component]
fn BoneTracker(_hooks: &mut Hooks, unit_id: EntityId) -> Element {
    let bones = entity::get_component(unit_id, bone_tracker_binders()).unwrap_or_default();
    let items: Vec<Element> = bones
        .iter()
        .enumerate()
        .map(|(index, name)| {
            BoneSelect::el(
                name.clone(),
                unit_id,
                index,
            )
        })
        .collect();

    FocusRoot::el([
        ScrollArea::el(
            ScrollAreaSizing::FitChildrenWidth,
            FlowColumn::el(items)
                .with_padding_even(STREET)
                .with(space_between_items(), 10.),
        )
    ])
}

const BONE_TRACKER_MIN_DISPLACEMENT: f32 = 0.1;
const BALL_RADIUS: f32 = 0.03;
const BONE_TRACKER_MAX_MARKERS: usize = 100;

fn spawn_ball(pos: Vec3) -> EntityId {
    Entity::new()
        .with_merge(make_transformable())
        .with_merge(make_sphere())
        .with(scale(), vec3(BALL_RADIUS, BALL_RADIUS, BALL_RADIUS))
        .with(color(), vec4(1.0, 0.0, 0.0, 1.0))
        .with(translation(), pos)
        .spawn()
}

impl BoneTracker {
    pub fn add_system() {
        query((
            local_to_world(),
            bone_tracker_enabled(),
            bone_tracker_index(),
            bone_tracker_markers(),
        ))
        .build()
        .each_frame(|units| {
            for (entity, (transform, enabled, mut index, mut markers)) in units {
                if !enabled {
                    if !markers.is_empty() {
                        for marker in markers {
                            entity::despawn(marker);
                        }
                        entity::set_components(
                            entity,
                            Entity::new()
                                .with(bone_tracker_index(), 0)
                                .with(bone_tracker_markers(), vec![]),
                        );
                    }

                    continue;
                }

                let (_, _, pos) = transform.to_scale_rotation_translation();

                let current = if let Some(current) =
                    markers.get(index as usize).or_else(|| markers.first())
                {
                    *current
                } else {
                    markers.push(spawn_ball(pos));
                    entity::set_components(
                        entity,
                        Entity::new()
                            .with(bone_tracker_index(), 0)
                            .with(bone_tracker_markers(), markers),
                    );
                    continue;
                };

                let last = entity::get_component(current, translation()).unwrap_or(pos);
                if last.distance(pos) < BONE_TRACKER_MIN_DISPLACEMENT {
                    continue;
                }

                if markers.len() < BONE_TRACKER_MAX_MARKERS {
                    markers.push(spawn_ball(pos));
                    entity::set_components(
                        entity,
                        Entity::new()
                            .with(bone_tracker_index(), markers.len() as u32 - 1)
                            .with(bone_tracker_markers(), markers),
                    );
                    continue;
                }

                index = (index + 1) % markers.len() as u32;

                let current = markers[index as usize];
                entity::set_component(entity, bone_tracker_index(), index);
                entity::set_component(current, translation(), pos);
            }
        });
    }

    pub fn setup(unit_id: EntityId) {
        let entities = entity::get_animation_binder_mask_entities(unit_id);
        let binders = entity::get_animation_binder_mask(unit_id);

        let mut available = Vec::with_capacity(binders.len());
        for (name, &entity) in binders.into_iter().zip(entities.iter()) {
            if !entity.is_null() {
                available.push((name, entity));
            }
        }

        available.sort_by_key(|x| x.0.clone());

        let trackable_entities: Vec<EntityId> = available.iter().map(|x| x.1).collect();
        let trackable_binders = available.into_iter().map(|x| x.0).collect();

        for &entity_id in trackable_entities.iter() {
            entity::add_components(
                entity_id,
                Entity::new()
                    .with(bone_tracker_enabled(), true)
                    .with_default(bone_tracker_index())
                    .with_default(bone_tracker_markers()),
            );
        }

        entity::add_components(
            unit_id,
            Entity::new()
                .with(bone_tracker_entities(), trackable_entities)
                .with(bone_tracker_binders(), trackable_binders),
        );
    }

    pub fn set_enabled_by_index(entity_id: EntityId, index: usize, enabled: bool) {
        let entities =
            entity::get_component(entity_id, bone_tracker_entities()).unwrap_or_default();
        let track = entities[index];
        entity::set_component(track, bone_tracker_enabled(), enabled);
    }
}
