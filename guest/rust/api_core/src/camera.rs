use once_cell::sync::Lazy;

use crate::{
    core::camera::components::active_camera,
    ecs::{query, Component, GeneralQuery},
    entity,
    global::EntityId,
    internal::generated::ambient_core::player::components::user_id,
};

#[cfg(feature = "client")]
mod client {
    use crate::{
        global::{EntityId, Ray, Vec2, Vec3},
        internal::{
            conversion::{FromBindgen, IntoBindgen},
            wit,
        },
    };

    /// Converts clip-space coordinates to a [Ray] in world space.
    ///
    /// To obtain clip-space coordinates, use [screen_to_clip_space].
    pub fn clip_position_to_world_ray(camera: EntityId, clip_space_position: Vec2) -> Ray {
        wit::client_camera::clip_position_to_world_ray(
            camera.into_bindgen(),
            clip_space_position.into_bindgen(),
        )
        .from_bindgen()
    }

    /// Converts a screen position (e.g. mouse position) to clip-space coordinates for the window.
    pub fn screen_to_clip_space(screen_position: Vec2) -> Vec2 {
        wit::client_camera::screen_to_clip_space(screen_position.into_bindgen()).from_bindgen()
    }

    /// Converts a screen position (e.g. mouse position) to a [Ray] in world space.
    pub fn screen_position_to_world_ray(camera: EntityId, screen_position: Vec2) -> Ray {
        wit::client_camera::screen_position_to_world_ray(
            camera.into_bindgen(),
            screen_position.into_bindgen(),
        )
        .from_bindgen()
    }

    /// Converts a world-space position to a screen position (e.g. mouse position).
    pub fn world_to_screen(camera: EntityId, world_position: Vec3) -> Vec2 {
        wit::client_camera::world_to_screen(camera.into_bindgen(), world_position.into_bindgen())
            .from_bindgen()
    }
}
#[cfg(feature = "client")]
pub use client::*;

/// Get the active camera (with filtering for a specific player, if required).
// TODO: consider moving this to the host
pub fn get_active(player_user_id: Option<&str>) -> Option<EntityId> {
    static QUERY: Lazy<GeneralQuery<Component<f32>>> = Lazy::new(|| query(active_camera()).build());

    QUERY
        .evaluate()
        .into_iter()
        .filter(|(id, _)| match player_user_id {
            Some(player_user_id) => {
                entity::get_component(*id, user_id()).is_some_and(|id| id == player_user_id)
            }
            None => true,
        })
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Less))
        .map(|(id, _)| id)
}
