use ambient_core::{camera::*, transform::*, ui_scene};
use ambient_ecs::{components, Networked, Store, SystemGroup};
use ambient_element::{element_component, Element, Hooks};
use ambient_std::shapes::BoundingBox;
use glam::{Quat, Vec3};
use winit::event::Event;

use crate::{free::free_camera_system, spherical::spherical_camera_system};

pub mod free;
pub mod spherical;

components!("camera", {
    @[Networked, Store]
    camera_movement_speed: f32,
});

pub fn init_all_components() {
    free::init_components();
    init_components();
    spherical::init_components();
}

pub fn assets_camera_systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new("assets_camera_systems", vec![Box::new(free_camera_system()), Box::new(spherical_camera_system())])
}

#[element_component]
pub fn UICamera(_: &mut Hooks) -> Element {
    Element::new()
        .init_default(local_to_world())
        .init_default(inv_local_to_world())
        .init(near(), -1.)
        .init(far(), 1.0)
        .init_default(orthographic())
        .init(orthographic_left(), 0.)
        .init(orthographic_right(), 100.)
        .init(orthographic_top(), 0.)
        .init(orthographic_bottom(), 100.)
        .init(orthographic_rect(), OrthographicRect { left: 0.0, right: 100., top: 0., bottom: 100. })
        .init_default(projection())
        .init_default(projection_view())
        .init_default(translation())
        .init_default(rotation())
        .init_default(orthographic_from_window())
        .init_default(ui_scene())
}

#[element_component]
pub fn LookatCamera(_: &mut Hooks, eye: Vec3, lookat: Vec3, up: Vec3) -> Element {
    Element::new()
        .init_default(local_to_world())
        .init_default(inv_local_to_world())
        .init(near(), 0.1)
        .init(fovy(), 1.0)
        .init(perspective_infinite_reverse(), ())
        .init(aspect_ratio(), 1.)
        .init(aspect_ratio_from_window(), ())
        .init_default(projection())
        .init_default(projection_view())
        .set(translation(), eye)
        .set(lookat_center(), lookat)
        .set(lookat_up(), up)
}

#[element_component]
pub fn FreeCamera(_: &mut Hooks, position: Vec3, rotation: Quat) -> Element {
    Element::new()
        .init_default(local_to_world())
        .init_default(inv_local_to_world())
        .init(near(), 0.1)
        .init(fovy(), 1.0)
        .init(perspective_infinite_reverse(), ())
        .init(aspect_ratio(), 1.)
        .init(aspect_ratio_from_window(), ())
        .init_default(projection())
        .init_default(projection_view())
        .set(ambient_core::transform::translation(), position)
        .set(ambient_core::transform::rotation(), rotation)
}

#[element_component]
pub fn FittedOrthographicCamera(_: &mut Hooks, eye: Vec3, lookat: Vec3, up: Vec3, fit: BoundingBox, aspect: f32) -> Element {
    Element::new().extend(Camera::fitted_ortographic(eye, lookat, up, fit, aspect).to_entity_data())
}
