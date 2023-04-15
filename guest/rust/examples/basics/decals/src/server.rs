use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::{quad, cube},
        rendering::{color, renderer_shader},
        transform::{lookat_center, rotation, scale, translation},
    },
    concepts::{make_perspective_infinite_reverse_camera},
    prelude::*,
};

#[main]
pub fn main() {
    Entity::new()
        .with_default(cube())
        .with(translation(), Vec3::Z)
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with_default(cast_shadows())
        .spawn();

    Entity::new()
        .with_default(quad())
        .with(scale(), Vec3::ONE * 10.)
        .spawn();


    Entity::new()
        .with_default(cube())
        .with(scale(), vec3(2., 2., 4.))
        .with(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .with(
            renderer_shader(),
            cb(move |assets, config| {
                DecalShaderKey { material_shader: PbrMaterialShaderKey.get(assets), lit: true, shadow_cascades: config.shadow_cascades }
                    .get(assets)
            }),
        )
        .init(material(), PbrMaterial::base_color_from_file(&assets, "assets/checkerboard.png").into())
        .spawn_static(world);

    let transparent = SharedMaterial::new(FlatMaterial::new(assets, vec4(0., 1., 0., 0.5), Some(true)));
    Cube.el()
        .with(scale(), vec3(2., 2., 4.))
        .with(rotation(), Quat::from_rotation_y(PI / 4.) * Quat::from_rotation_z(PI / 4.))
        .with(material(), transparent)
        .spawn_static(world);

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .with(active_camera(), 0.)
        .with(main_scene(), ())
        .spawn(world);
}
