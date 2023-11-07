#[cfg(feature = "client")]
mod client {
    use crate::{
        core::camera::components::active_camera,
        ecs::{query, Component, GeneralQuery},
        entity,
        global::{EntityId, Ray, Vec2, Vec3},
        internal::{
            conversion::{FromBindgen, IntoBindgen},
            generated::ambient_core::{app::components::main_scene, player::components::user_id},
            wit,
        },
        player,
    };
    use once_cell::sync::Lazy;

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
    pub fn world_to_screen(camera: EntityId, world_position: Vec3) -> Vec3 {
        wit::client_camera::world_to_screen(camera.into_bindgen(), world_position.into_bindgen())
            .from_bindgen()
    }

    /// Get the active camera.
    // TODO: consider moving this to the host
    pub fn get_active() -> Option<EntityId> {
        static QUERY: Lazy<GeneralQuery<Component<f32>>> =
            Lazy::new(|| query(active_camera()).requires(main_scene()).build());

        let local_user_id = entity::get_component(player::get_local(), user_id());

        QUERY
            .evaluate()
            .into_iter()
            .filter(|(id, _)| {
                if let Some(local_user_id) = &local_user_id {
                    if let Some(cam_user_id) = entity::get_component(*id, user_id()) {
                        cam_user_id == *local_user_id
                    } else {
                        // The camera is considered global, as it doesn't have a user_id attached
                        true
                    }
                } else {
                    // No user_id was supplied, so all cameras are considered
                    true
                }
            })
            .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Less))
            .map(|(id, _)| id)
    }
}
#[cfg(feature = "client")]
pub use client::*;
