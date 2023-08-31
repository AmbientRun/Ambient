use super::EntityId;
use crate::{
    core::transform::components::local_to_world,
    ecs,
    internal::{conversion::FromBindgen, wit},
    prelude::World,
};
use glam::{vec3, Vec3};

#[derive(Debug, Clone, Copy, Default)]
/// Ray represented by an origin and a direction
pub struct Ray {
    /// Origin of the ray
    pub origin: Vec3,
    /// Direction of the ray
    pub dir: Vec3,
}
impl Ray {
    /// This creates a ray from a cameras view matrix (i.e. from `local_to_world` of a camera entity).
    pub fn from_camera_view_matrix(world: &World, camera: EntityId) -> ecs::Result<Ray> {
        let mat4 = world.get_component(camera, local_to_world())?;
        let origin = mat4.project_point3(Vec3::ZERO);
        let end = mat4.project_point3(vec3(0., 0., 1.));
        let dir = (end - origin).normalize();
        Ok(Ray { origin, dir })
    }
}

impl FromBindgen for wit::types::Ray {
    type Item = Ray;
    fn from_bindgen(self) -> Self::Item {
        Ray {
            origin: self.origin.from_bindgen(),
            dir: self.dir.from_bindgen(),
        }
    }
}
