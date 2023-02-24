use ambient_app::{App, AppBuilder};
use ambient_core::{camera::active_camera, main_scene, transform::*};
use ambient_element::ElementComponentExt;
use ambient_primitives::Cube;
use ambient_renderer::{cast_shadows, color};
use ambient_std::math::SphericalCoords;
use glam::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let size = 5;

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                Cube.el()
                    .set(color(), (Vec3::ONE - vec3(x as f32, y as f32, z as f32) / (size - 1) as f32).extend(1.))
                    .set(translation(), vec3(x as f32, y as f32, z as f32))
                    .set(scale(), Vec3::ONE * 0.4)
                    .set_default(cast_shadows())
                    .spawn_static(world);
            }
        }
    }

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
