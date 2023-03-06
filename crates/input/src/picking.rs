use ambient_core::{
    camera::{get_active_camera, screen_ray},
    mouse_position,
    transform::local_to_world,
    ui_scene, window_physical_size,
};
use ambient_ecs::{components, query, Debuggable, Description, Entity, EntityId, FnSystem, Name, Networked, Resource, Store, SystemGroup};
use ambient_std::shapes::{RayIntersectable, AABB};
use glam::{Vec2, Vec3};

components!("input", {
    @[Resource]
    picker_intersecting: Option<PickerIntersection>,

    @[Debuggable, Networked, Store, Name["Mouse pickable min"], Description["This entity can be clicked by the mouse, and this component defines the min AABB bound of the click area."]]
    mouse_pickable_min: Vec3,
    @[Debuggable, Networked, Store, Name["Mouse pickable max"], Description["This entity can be clicked by the mouse, and this component defines the max AABB bound of the click area."]]
    mouse_pickable_max: Vec3,
    mouse_pickable: AABB,
    @[Debuggable, Networked, Store, Name["Mouse over"], Description["A mouse cursor is currently over this entity."]]
    mouse_over: bool,
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
            Box::new(FnSystem::new(|world, _| {
                let window_size = world.resource(window_physical_size());
                let mouse_position = *world.resource(mouse_position());
                let mut mouse_origin = -Vec2::ONE + (mouse_position / window_size.as_vec2()) * 2.;
                mouse_origin.y = -mouse_origin.y;
                let camera = match get_active_camera(world, ui_scene()) {
                    Some(cam) => cam,
                    None => return,
                };
                let ray = screen_ray(world, camera, mouse_origin).unwrap_or_default();

                let prev_intersecting = *world.resource(picker_intersecting());

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
                        world.add_component(prev, mouse_over(), false).unwrap();
                    }
                    if let Some(new) = intersecting_entity {
                        world.add_component(new, mouse_over(), true).unwrap();
                    }
                }
                *world.resource_mut(picker_intersecting()) = intersecting;
            })),
        ],
    )
}
