use ambient_core::{
    camera::{clip_position_to_world_ray, get_active_camera},
    player::{local_user_id, user_id},
    transform::local_to_world,
    ui_scene,
    window::{cursor_position, window_logical_size},
};
use ambient_ecs::{
    components,
    generated::input::components::{mouse_over_distance, mouse_over_entity},
    query, Debuggable, Entity, EntityId, SystemGroup,
};
use ambient_native_std::shapes::{RayIntersectable, AABB};
use glam::Vec2;

pub use ambient_ecs::generated::input::components::{
    is_mouse_over, mouse_pickable_max, mouse_pickable_min,
};

components!("input", {

    @[Debuggable]
    mouse_pickable: AABB,
});

pub fn resources() -> Entity {
    Entity::new()
        .with(mouse_over_entity(), EntityId::null())
        .with(mouse_over_distance(), f32::MAX)
}

pub fn frame_systems() -> SystemGroup {
    SystemGroup::new(
        "picking",
        vec![
            query((
                mouse_pickable_min().changed(),
                mouse_pickable_max().changed(),
            ))
            .to_system(|q, world, qs, _| {
                for (id, (min, max)) in q.collect_cloned(world, qs) {
                    world
                        .add_component(id, mouse_pickable(), AABB { min, max })
                        .unwrap();
                }
            }),
            query((window_logical_size(), cursor_position())).to_system(|q, world, qs, _| {
                for (id, (window_size, mouse_position)) in q.collect_cloned(world, qs) {
                    let mut mouse_origin =
                        -Vec2::ONE + (mouse_position / window_size.as_vec2()) * 2.;
                    mouse_origin.y = -mouse_origin.y;
                    let camera = match get_active_camera(
                        world,
                        ui_scene(),
                        world
                            .get_ref(id, user_id())
                            .ok()
                            .or_else(|| world.resource_opt(local_user_id())),
                    ) {
                        Some(cam) => cam,
                        None => return,
                    };
                    let ray =
                        clip_position_to_world_ray(world, camera, mouse_origin).unwrap_or_default();

                    let prev_intersecting_entity =
                        world.get(id, mouse_over_entity()).unwrap_or_default();

                    let mut intersecting_entity = EntityId::null();
                    let mut intersecting_dist = 0.;
                    for (id2, (pickable, local_to_world)) in
                        query((mouse_pickable(), local_to_world())).iter(world, None)
                    {
                        if local_to_world.is_nan() {
                            continue;
                        }
                        let ray = ray.transform(local_to_world.inverse());
                        if let Some(dist) = pickable.ray_intersect(ray) {
                            if intersecting_entity.is_null() || dist < intersecting_dist {
                                intersecting_entity = id2;
                                intersecting_dist = dist;
                            }
                        }
                    }
                    if prev_intersecting_entity != intersecting_entity {
                        if !prev_intersecting_entity.is_null() {
                            if let Ok(prev_mouse_over) =
                                world.get(prev_intersecting_entity, is_mouse_over())
                            {
                                if prev_mouse_over > 0 {
                                    world
                                        .add_component(
                                            prev_intersecting_entity,
                                            is_mouse_over(),
                                            prev_mouse_over - 1,
                                        )
                                        .unwrap();
                                }
                            }
                        }
                        if !intersecting_entity.is_null() {
                            let new_mouse_over = world
                                .get(intersecting_entity, is_mouse_over())
                                .unwrap_or_default();
                            world
                                .add_component(
                                    intersecting_entity,
                                    is_mouse_over(),
                                    new_mouse_over + 1,
                                )
                                .unwrap();
                        }
                    }
                    world
                        .add_component(id, mouse_over_entity(), intersecting_entity)
                        .unwrap();
                    world
                        .add_component(id, mouse_over_distance(), intersecting_dist)
                        .unwrap();
                }
            }),
        ],
    )
}
