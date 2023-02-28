use ambient_core::{camera::*, transform::*};
use ambient_ecs::{components, query_mut, Entity, SystemGroup};
use derive_more::Display;
use glam::vec2;
use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent};

use super::camera_movement_speed;

components!("camera", {
    free_camera: FreeCamera,
});

#[derive(Debug, Default, Display, Clone)]
#[display(fmt = "{self:?}")]
pub struct FreeCamera {
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    orientation: glam::Vec2,
}

pub fn new(position: glam::Vec3, orientation: glam::Vec2) -> Entity {
    let free = FreeCamera { orientation, ..Default::default() };
    Entity::new()
        .set_default(local_to_world())
        .set_default(inv_local_to_world())
        .set(near(), 0.1)
        .set(fovy(), 1.0)
        .set(perspective_infinite_reverse(), ())
        .set(aspect_ratio(), 1.)
        .set(aspect_ratio_from_window(), ())
        .set_default(projection())
        .set_default(projection_view())
        .set(translation(), position)
        .set_default(rotation())
        .set(free_camera(), free)
        .set(camera_movement_speed(), 0.1)
}

pub fn free_camera_system() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new(
        "free_camera_system",
        vec![query_mut((free_camera(), translation(), rotation(), camera_movement_speed(), far()), ()).to_system(|q, world, qs, event| {
            for (_, (free_camera, translation, rotation, speed, far), ()) in q.iter(world, qs) {
                match event {
                    Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                        let speed = 0.01;
                        free_camera.orientation += vec2(delta.0 as f32, delta.1 as f32) * speed;
                    }
                    Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                        let is_pressed = input.state == ElementState::Pressed;
                        if let Some(keycode) = input.virtual_keycode {
                            match keycode {
                                VirtualKeyCode::E => free_camera.is_up_pressed = is_pressed,
                                VirtualKeyCode::Q => free_camera.is_down_pressed = is_pressed,
                                VirtualKeyCode::W | VirtualKeyCode::Up => free_camera.is_forward_pressed = is_pressed,
                                VirtualKeyCode::A | VirtualKeyCode::Left => free_camera.is_left_pressed = is_pressed,
                                VirtualKeyCode::S | VirtualKeyCode::Down => free_camera.is_backward_pressed = is_pressed,
                                VirtualKeyCode::D | VirtualKeyCode::Right => free_camera.is_right_pressed = is_pressed,
                                VirtualKeyCode::R => *speed *= 2.0,
                                VirtualKeyCode::F => *speed /= 2.0,
                                VirtualKeyCode::T => *far *= 2.0,
                                VirtualKeyCode::G => *far /= 2.0,
                                _ => {}
                            }
                        }
                    }
                    Event::RedrawRequested(_) => {
                        let mut velocity = glam::Vec3::ZERO;
                        if free_camera.is_up_pressed {
                            velocity += glam::Vec3::Z;
                        }
                        if free_camera.is_down_pressed {
                            velocity -= glam::Vec3::Z;
                        }
                        if free_camera.is_forward_pressed {
                            velocity += (*rotation) * glam::Vec3::Z;
                        }
                        if free_camera.is_backward_pressed {
                            velocity -= (*rotation) * glam::Vec3::Z;
                        }
                        if free_camera.is_left_pressed {
                            velocity -= (*rotation) * glam::Vec3::X;
                        }
                        if free_camera.is_right_pressed {
                            velocity += (*rotation) * glam::Vec3::X;
                        }
                        *translation += velocity * (*speed);

                        *rotation =
                            glam::Quat::from_rotation_z(free_camera.orientation.x) * glam::Quat::from_rotation_x(free_camera.orientation.y);
                        // *rotation = glam::Quat::from_rotation_z(free_camera.orientation.x)
                        //     * glam::Quat::from_rotation_y(free_camera.orientation.y);
                    }
                    _ => {}
                }
            }
        })],
    )
}
