use glam::{vec3, vec4, Vec3, Vec4};
use kiwi_app::AppBuilder;
use kiwi_core::{
    camera::active_camera, main_scene, transform::{scale, translation}
};
use kiwi_ecs::World;
use kiwi_element::ElementComponentExt;
use kiwi_primitives::{Cube, Quad};
use kiwi_renderer::{cast_shadows, color, outline};
use kiwi_std::math::SphericalCoords;

fn init(world: &mut World) {
    Cube.el()
        .set(color(), vec4(0.5, 0.5, 0.5, 1.))
        .set(translation(), Vec3::Z)
        .set_default(cast_shadows())
        .set(outline(), Vec4::ONE)
        .spawn_static(world);
    Quad.el().set(scale(), Vec3::ONE * 10.).spawn_static(world);

    kiwi_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    AppBuilder::simple().run_world(init);
}
