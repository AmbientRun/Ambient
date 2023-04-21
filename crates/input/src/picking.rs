use ambient_core::{
    camera::{get_active_camera, clip_space_ray},
    player::{local_user_id, user_id},
    transform::local_to_world,
    ui_scene,
    window::{cursor_position, window_logical_size},
};
use ambient_ecs::{components, query, Debuggable, Entity, EntityId, MaybeResource, SystemGroup};
use ambient_std::shapes::{RayIntersectable, AABB};
use glam::Vec2;

pub use ambient_ecs::generated::components::core::input::{mouse_over, mouse_pickable_max, mouse_pickable_min};

components!("input", {
    @[MaybeResource, Debuggable]
    picker_intersecting: Option<PickerIntersection>,

    @[Debuggable]
    mouse_pickable: AABB,
});

#[derive(Debug, Clone, Copy)]
pub struct PickerIntersection {
    pub entity: EntityId,
    pub distance: f32,
}

pub fn resources() -> Entity {
    Entity::new().with_default(picker_intersecting())
}

pub fn frame_systems() -> SystemGroup {
    SystemGroup::new(
        "picking",
        vec![
            query((mouse_pickable_min().changed(), mouse_pickable_max().changed())).to_system(|q, world, qs, _| {
                for (id, (min, max)) in q.collect_cloned(world, qs) {
                    world.add_component(id, mouse_pickable(), AABB { min, max }).unwrap();
                }
            }),
            query((window_logical_size(), cursor_position())).to_system(|q, world, qs, _| {
                for (id, (window_size, mouse_position)) in q.collect_cloned(world, qs) {
                    let mut mouse_origin = -Vec2::ONE + (mouse_position / window_size.as_vec2()) * 2.;
                    mouse_origin.y = -mouse_origin.y;
                    let camera = match get_active_camera(
                        world,
                        ui_scene(),
                        world.get_ref(id, user_id()).ok().or_else(|| world.resource_opt(local_user_id())),
                    ) {
                        Some(cam) => cam,
                        None => return,
                    };
                    let ray = clip_space_ray(world, camera, mouse_origin).unwrap_or_default();

                    let prev_intersecting = world.get(id, picker_intersecting()).unwrap_or_default();

                    let mut intersecting: Option<PickerIntersection> = None;
                    for (id2, (pickable, local_to_world)) in query((mouse_pickable(), local_to_world())).iter(world, None) {
                        if local_to_world.is_nan() {
                            continue;
                        }
                        let ray = ray.transform(local_to_world.inverse());
                        if let Some(dist) = pickable.ray_intersect(ray) {
                            if intersecting.is_none() || dist < intersecting.as_ref().unwrap().distance {
                                intersecting = Some(PickerIntersection { entity: id2, distance: dist });
                            }
                        }
                    }
                    let prev_intersecting_entity = prev_intersecting.map(|x| x.entity);
                    let intersecting_entity = intersecting.map(|x| x.entity);
                    if prev_intersecting_entity != intersecting_entity {
                        if let Some(prev) = prev_intersecting_entity {
                            if let Ok(prev_mouse_over) = world.get(prev, mouse_over()) {
                                if prev_mouse_over > 0 {
                                    world.add_component(prev, mouse_over(), prev_mouse_over - 1).unwrap();
                                }
                            }
                        }
                        if let Some(new) = intersecting_entity {
                            let new_mouse_over = world.get(new, mouse_over()).unwrap_or_default();
                            world.add_component(new, mouse_over(), new_mouse_over + 1).unwrap();
                        }
                    }
                    world.add_component(id, picker_intersecting(), intersecting).unwrap();
                }
            }),
        ],
    )
}
