use std::sync::Arc;

use glam::*;
use kiwi_app::{gpu, AppBuilder};
use kiwi_cameras::UICamera;
use kiwi_core::camera::active_camera;
use kiwi_ecs::World;
use kiwi_element::{ElementComponentExt, ElementTree};
use kiwi_gpu::texture::Texture;
use kiwi_ui::{
    layout::{height, width}, *
};

fn init(world: &mut World) {
    ElementTree::new(
        world,
        Image {
            texture: Some(Arc::new(
                Arc::new(Texture::new_single_color_texture(world.resource(gpu()).clone(), uvec4(255, 200, 200, 255)))
                    .create_view(&Default::default()),
            )),
        }
        .el()
        .set(width(), 200.)
        .set(height(), 200.),
    );

    UICamera.el().set(active_camera(), 0.).spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().run_world(init);
}
