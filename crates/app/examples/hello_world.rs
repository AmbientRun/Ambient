use elements_app::AppBuilder;
use elements_core::{
    camera::active_camera, main_scene, transform::{scale, translation}
};
use elements_ecs::World;
use elements_element::ElementComponentExt;
use elements_primitives::{Cube, Quad};
use elements_renderer::{cast_shadows, color, outline};
use elements_std::math::SphericalCoords;
use glam::{vec3, vec4, Vec3, Vec4};

fn init(world: &mut World) {
    Cube.el()
        .set(color(), vec4(0.5, 0.5, 0.5, 1.))
        .set(translation(), Vec3::Z)
        .set_default(cast_shadows())
        .set(outline(), Vec4::ONE)
        .spawn_static(world);
    Quad.el().set(scale(), Vec3::ONE * 10.).spawn_static(world);

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    AppBuilder::simple().run_world(init);
}
