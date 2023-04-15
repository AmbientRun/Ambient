use crate::{
    global::{Vec2, Ray, EntityId},
    internal::{
        wit,
        conversion::{FromBindgen, IntoBindgen},
    },
};

// #[cfg(feature = "client")]
#[allow(missing_docs)]
pub fn screen_ray(camera: EntityId, screen_space_pos: Vec2) -> Ray {
    wit::camera::screen_ray(camera.into_bindgen(), screen_space_pos.into_bindgen()).from_bindgen()
}
