use std::f32::consts::TAU;

use ambient_api::prelude::*;

pub struct CameraState {
    radius: f32,
    pivot: Vec3,
    position: Vec3,
    rotation: Vec2,
}
impl Default for CameraState {
    fn default() -> Self {
        CameraState {
            radius: 15.,
            pivot: vec3(0., 0., 8.),
            position: Vec3::ZERO,
            rotation: vec2(180_f32.to_radians(), 35_f32.to_radians()),
        }
    }
}
impl CameraState {
    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self
    }
    pub fn get_yaw(&self) -> f32 {
        self.rotation.x
    }
    pub fn get_transform_rotation(&self) -> Quat {
        Quat::from_rotation_z(self.rotation.x) * Quat::from_rotation_x(self.rotation.y)
    }
    pub fn get_transform(&self) -> (Vec3, Quat) {
        let rotation = self.get_transform_rotation();
        let position = self.position + self.pivot + rotation * (Vec3::Y * self.radius);
        (position, rotation)
    }
    pub fn zoom(&mut self, delta: f32) -> &mut Self {
        self.radius = f32::clamp(self.radius + delta, 5., 30.);
        self
    }
    pub fn rotate(&mut self, rotation: Vec2) -> &mut Self {
        self.rotation += rotation;
        while self.rotation.x < 0. {
            self.rotation.x += TAU;
        }
        while self.rotation.x > TAU {
            self.rotation.x -= TAU;
        }
        self.rotation.y = self
            .rotation
            .y
            .clamp(-20_f32.to_radians(), 40_f32.to_radians());
        self
    }
}
