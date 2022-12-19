use elements_core::{
    camera::{get_active_camera, screen_ray}, mouse_position, transform::local_to_world, ui_scene, window
};
use elements_ecs::{components, query, EntityData, EntityId, FnSystem, SystemGroup, World};
use elements_std::{
    events::EventDispatcher, shapes::{RayIntersectable, AABB}
};
use glam::{uvec2, Vec2};
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};

components!("input", {
    picker_intersecting: Option<PickerIntersection>,

    mouse_pickable: AABB,
    on_mouse_input: EventDispatcher<dyn Fn(&mut World, EntityId, ElementState, MouseButton) + Send + Sync>,
    on_mouse_enter: EventDispatcher<dyn Fn(&mut World, EntityId) + Send + Sync>,
    on_mouse_leave: EventDispatcher<dyn Fn(&mut World, EntityId) + Send + Sync>,
    on_mouse_hover: EventDispatcher<dyn Fn(&mut World, EntityId) + Send + Sync>,
    on_mouse_wheel: EventDispatcher<dyn Fn(&mut World, EntityId, MouseScrollDelta) + Sync + Send>,
});

#[derive(Debug, Clone, Copy)]
pub struct PickerIntersection {
    pub entity: EntityId,
    pub distance: f32,
}

pub fn resources() -> EntityData {
    EntityData::new().set_default(picker_intersecting())
}

pub fn frame_systems() -> SystemGroup {
    SystemGroup::new(
        "picking",
        vec![Box::new(FnSystem::new(|world, _| {
            let window_size = {
                let size = world.resource(window()).inner_size();
                uvec2(size.width, size.height)
            };
            let mouse_position = *world.resource(mouse_position());
            let mut mouse_origin = -Vec2::ONE + (mouse_position / window_size.as_vec2()) * 2.;
            mouse_origin.y = -mouse_origin.y;
            let camera = match get_active_camera(world, ui_scene()) {
                Some(cam) => cam,
                None => return,
            };
            let ray = screen_ray(world, camera, mouse_origin).unwrap_or_default();

            let mut on_leaves = Vec::new();
            let mut on_enters = Vec::new();
            let mut on_hovers = Vec::new();
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
                    if let Ok(on_leave) = world.get_ref(prev, on_mouse_leave()) {
                        on_leaves.push((on_leave.clone(), prev));
                    }
                }
                if let Some(new) = intersecting_entity {
                    if let Ok(on_enter) = world.get_ref(new, on_mouse_enter()) {
                        on_enters.push((on_enter.clone(), new));
                    }
                }
            }
            if let Some(new) = intersecting_entity {
                if let Ok(on_hover) = world.get_ref(new, on_mouse_hover()) {
                    on_hovers.push((on_hover.clone(), new));
                }
            }
            *world.resource_mut(picker_intersecting()) = intersecting;

            for (on_leave, ent) in on_leaves.into_iter() {
                for handler in on_leave.iter() {
                    handler(world, ent);
                }
            }
            for (on_enter, ent) in on_enters.into_iter() {
                for handler in on_enter.iter() {
                    handler(world, ent);
                }
            }
            for (on_hover, ent) in on_hovers.into_iter() {
                for handler in on_hover.iter() {
                    handler(world, ent);
                }
            }
        }))],
    )
}

pub fn picking_winit_event_system() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new(
        "picking_winit_event_system",
        vec![Box::new(FnSystem::new(|world, event| match event {
            Event::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } => {
                let intersecting = *world.resource(picker_intersecting());
                if let Some(intersecting) = intersecting {
                    if let Ok(on_mouse_input) = world.get_ref(intersecting.entity, on_mouse_input()).cloned() {
                        for handler in on_mouse_input.iter() {
                            handler(world, intersecting.entity, *state, *button);
                        }
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } => {
                let intersecting = *world.resource(picker_intersecting());
                if let Some(intersecting) = intersecting {
                    if let Ok(on_mouse_wheel) = world.get_ref(intersecting.entity, on_mouse_wheel()).cloned() {
                        for handler in on_mouse_wheel.iter() {
                            handler(world, intersecting.entity, *delta);
                        }
                    }
                }
            }
            _ => {}
        }))],
    )
}
