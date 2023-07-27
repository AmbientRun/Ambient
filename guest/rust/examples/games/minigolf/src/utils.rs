use std::f32::consts::TAU;

use ambient_api::prelude::*;

use crate::ambient_example_minigolf::components::{
    player_camera_pivot, player_camera_position, player_camera_radius, player_camera_rotation,
};

pub struct CameraState(pub EntityId);
impl CameraState {
    pub fn set_position(&self, position: Vec3) -> &Self {
        entity::set_component(self.0, player_camera_position(), position);
        self
    }
    pub fn get_yaw(&self) -> f32 {
        entity::get_component(self.0, player_camera_rotation())
            .unwrap_or_default()
            .x
    }
    pub fn get_transform_rotation(&self) -> Quat {
        let rotation = entity::get_component(self.0, player_camera_rotation()).unwrap_or_default();
        Quat::from_rotation_z(rotation.x) * Quat::from_rotation_x(rotation.y)
    }
    pub fn get_transform(&self) -> (Vec3, Quat) {
        let position = entity::get_component(self.0, player_camera_position()).unwrap_or_default();
        let pivot = entity::get_component(self.0, player_camera_pivot()).unwrap_or_default();
        let rotation = self.get_transform_rotation();
        let radius = entity::get_component(self.0, player_camera_radius()).unwrap_or_default();
        (position + pivot + rotation * (Vec3::Y * radius), rotation)
    }
    pub fn zoom(&self, delta: f32) -> &Self {
        entity::mutate_component(self.0, player_camera_radius(), |radius| {
            *radius = f32::clamp(*radius + delta, 5., 30.);
        });
        self
    }
    pub fn rotate(&self, rotation: Vec2) -> &Self {
        entity::mutate_component(self.0, player_camera_rotation(), |rot| {
            *rot += rotation;
            while rot.x < 0. {
                rot.x += TAU;
            }
            while rot.x > TAU {
                rot.x -= TAU;
            }
            rot.y = rot.y.clamp(-20_f32.to_radians(), 40_f32.to_radians());
        });
        self
    }
}

pub fn hsv_to_rgb([h, s, v]: &[f32; 3]) -> Vec3 {
    let c = v * s;
    let p = (h / 60.) % 6.;
    let x = c * (1.0 - ((p % 2.) - 1.).abs());
    let m = Vec3::ONE * (v - c);

    m + match p.trunc() as i32 {
        0 => vec3(c, x, 0.),
        1 => vec3(x, c, 0.),
        2 => vec3(0., c, x),
        3 => vec3(0., x, c),
        4 => vec3(x, 0., c),
        5 => vec3(c, 0., x),
        _ => vec3(0., 0., 0.),
    }
}
