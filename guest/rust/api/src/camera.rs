use crate::{
    global::{Vec2, Ray},
    internal::{
        wit,
        conversion::{FromBindgen, IntoBindgen},
    },
};

// #[cfg(feature = "client")]
#[allow(missing_docs)]
pub fn screen_ray(screen_space_pos: Vec2) -> Ray {
    wit::camera::screen_ray(screen_space_pos.into_bindgen()).from_bindgen()
}