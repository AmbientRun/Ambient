use ambient_api::{
    core::{
        app::components::main_scene,
        camera::{
            components::aspect_ratio_from_window,
            concepts::make_perspective_infinite_reverse_camera,
        },
        messages::Frame,
        model::components::model_from_url,
        player::components::{is_player, local_user_id, user_id},
        transform::components::{lookat_target, rotation, translation},
    },
    entity::add_component,
    prelude::*,
};

use packages::base_assets;

#[main]
fn main() {}
