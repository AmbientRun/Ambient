use std::sync::Arc;

use elements_app::{gpu, App};
use elements_cameras::UICamera;
use elements_core::camera::active_camera;
use elements_ecs::World;
use elements_element::{ElementComponentExt, ElementTree};
use elements_gpu::texture::Texture;
use elements_ui::{
    layout::{height, width},
    *,
};
use glam::*;

fn init(world: &mut World) {
    ElementTree::new(
        world,
        Image {
            texture: Some(Arc::new(
                Arc::new(Texture::new_single_color_texture(
                    world.resource(gpu()).clone(),
                    uvec4(255, 200, 200, 255),
                ))
                .create_view(&Default::default()),
            )),
        }
        .el()
        .set(width(), 200.)
        .set(height(), 200.),
    );

    UICamera
        .el()
        .set(active_camera(), 0.)
        .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    App::run_ui(init);
}
