use derive_more::Display;
use kiwi_ecs::{components, query_mut, EntityData, SystemGroup};
use kiwi_std::math::SphericalCoords;
use winit::event::{DeviceEvent, ElementState, Event, MouseScrollDelta, VirtualKeyCode, WindowEvent};

use super::*;

components!("camera", {
    spherical_camera: SphericalCamera,
});

#[derive(Debug, Default, Display, Clone)]
#[display(fmt = "{self:?}")]
pub struct SphericalCamera {
    is_rotating: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    orientation: SphericalCoords,
}
impl SphericalCamera {
    fn translation(&self, lookat_center: glam::Vec3) -> glam::Vec3 {
        lookat_center + glam::Vec3::from(self.orientation)
    }
}

pub fn new(lookat: glam::Vec3, orientation: SphericalCoords) -> EntityData {
    let spherical = SphericalCamera { orientation, ..Default::default() };
    EntityData::new()
        .set_default(local_to_world())
        .set_default(inv_local_to_world())
        .set(near(), 0.1)
        .set(fovy(), 1.0)
        .set(perspective_infinite_reverse(), ())
        .set(aspect_ratio(), 1.)
        .set(aspect_ratio_from_window(), ())
        .set_default(projection())
        .set_default(projection_view())
        .set(translation(), spherical.translation(lookat))
        .set(lookat_center(), lookat)
        .set(lookat_up(), glam::vec3(0., 0., 1.))
        .set(spherical_camera(), spherical)
        .set(camera_movement_speed(), 0.1)
}

pub fn spherical_camera_system() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new(
        "spherical_camera_system",
        vec![query_mut((spherical_camera(), translation(), lookat_center(), camera_movement_speed()), ()).to_system(
            |q, world, qs, event| {
                for (_, (spherical_camera, translation, lookat_center, speed), ()) in q.iter(world, qs) {
                    match event {
                        Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                            if spherical_camera.is_rotating {
                                let speed = 0.01;
                                spherical_camera.orientation.phi += delta.0 as f32 * speed;
                                spherical_camera.orientation.theta -= delta.1 as f32 * speed;
                            }
                        }
                        Event::WindowEvent { event, .. } => {
                            match event {
                                WindowEvent::KeyboardInput { input, .. } => {
                                    let is_pressed = input.state == ElementState::Pressed;
                                    if let Some(keycode) = input.virtual_keycode {
                                        match keycode {
                                            VirtualKeyCode::E => spherical_camera.is_up_pressed = is_pressed,
                                            VirtualKeyCode::Q => spherical_camera.is_down_pressed = is_pressed,
                                            VirtualKeyCode::W | VirtualKeyCode::Up => spherical_camera.is_forward_pressed = is_pressed,
                                            VirtualKeyCode::A | VirtualKeyCode::Left => spherical_camera.is_left_pressed = is_pressed,
                                            VirtualKeyCode::S | VirtualKeyCode::Down => spherical_camera.is_backward_pressed = is_pressed,
                                            VirtualKeyCode::D | VirtualKeyCode::Right => spherical_camera.is_right_pressed = is_pressed,
                                            VirtualKeyCode::R => *speed *= 2.0,
                                            VirtualKeyCode::F => *speed /= 2.0,
                                            VirtualKeyCode::Space => spherical_camera.is_rotating = is_pressed,
                                            _ => {}
                                        }
                                    }
                                }
                                WindowEvent::MouseWheel { delta, .. } => {
                                    spherical_camera.orientation.radius *= 1.
                                        + match delta {
                                            MouseScrollDelta::LineDelta(_, y) => y * 0.05,
                                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                                        }
                                }
                                WindowEvent::MouseInput { .. } => {
                                    // spherical_camera.is_rotating = state == &ElementState::Pressed;
                                }
                                _ => {}
                            }
                        }
                        Event::MainEventsCleared => {
                            let mut velocity = glam::Vec3::ZERO;
                            let rotation = glam::Quat::from_rotation_z(spherical_camera.orientation.phi);
                            if spherical_camera.is_up_pressed {
                                velocity += glam::Vec3::Z;
                            }
                            if spherical_camera.is_down_pressed {
                                velocity -= glam::Vec3::Z;
                            }
                            if spherical_camera.is_forward_pressed {
                                velocity -= rotation * glam::Vec3::X;
                            }
                            if spherical_camera.is_backward_pressed {
                                velocity += rotation * glam::Vec3::X;
                            }
                            if spherical_camera.is_left_pressed {
                                velocity += rotation * glam::Vec3::Y;
                            }
                            if spherical_camera.is_right_pressed {
                                velocity -= rotation * glam::Vec3::Y;
                            }
                            *lookat_center += velocity * (*speed);
                            *translation = spherical_camera.translation(*lookat_center);
                        }
                        _ => {}
                    }
                }
            },
        )],
    )
}
