use crate::internal::{conversion::FromBindgen, wit};
use glam::Vec3;

/// Ray represented by an origin and a direction
pub struct Ray {
    /// Origin of the ray
    pub origin: Vec3,
    /// Direction of the ray
    pub dir: Vec3,
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
