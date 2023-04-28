use std::sync::Arc;

use ambient_app::{gpu, AmbientWindow, AppBuilder};
use ambient_cameras::UICamera;
use ambient_element::{ElementComponentExt, ElementTree};
use ambient_gpu::texture::Texture;
use ambient_ui_native::{
    layout::{height, width},
    *,
};
use glam::*;

async fn init(app: &mut AmbientWindow) {
    let world = &mut app.world;
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
        .with(width(), 200.)
        .with(height(), 200.),
    );

    UICamera.el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
