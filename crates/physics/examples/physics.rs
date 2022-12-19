use elements_app::{gpu, App};
use elements_core::{asset_cache, camera::active_camera, main_scene, transform::scale, FixedTimestepSystem};
use elements_ecs::{FnSystem, World};
use elements_element::ElementComponentExt;
use elements_physics::physx::{physics, rigid_dynamic, rigid_static, sync_ecs_physics};
use elements_primitives::{Cube, Quad};
use elements_renderer::color;
use elements_std::math::SphericalCoords;
use glam::*;
use physxx::*;
use rand::random;

async fn init(world: &mut World) -> PxSceneRef {
    let _gpu = world.resource(gpu()).clone();
    let _assets = world.resource(asset_cache()).clone();

    let physics = world.resource(physics()).clone();
    let scene = {
        let mut scene_desc = PxSceneDesc::new(&physics.physics);
        scene_desc.set_cpu_dispatcher(&physics.dispatcher);
        scene_desc.set_gravity(vec3(0., 0., -9.82));
        PxSceneRef::new(&physics.physics, &scene_desc)
    };

    // Ground plane
    let physics_material = PxMaterial::new(&physics.physics, 0.5, 0.5, 0.6);
    let ground_static = PxRigidStaticRef::new_plane(physics.physics, vec3(0., 0., 1.), 0., &physics_material);
    scene.add_actor(&ground_static);
    Quad.el().set(scale(), Vec3::ONE * 40.).set(color(), vec4(0.5, 0.5, 0.5, 1.)).set(rigid_static(), ground_static).spawn_static(world);

    // Boxes
    for _ in 0..10 {
        let actor = PxRigidDynamicRef::new_with_geometry(
            &physics.physics,
            &PxTransform::from_translation(vec3(10. * (random::<f32>() * 2. - 1.), 10. * (random::<f32>() * 2. - 1.), 10.)),
            &PxBoxGeometry::new(1., 1., 1.),
            &physics_material,
            10.,
            &PxTransform::identity(),
        );
        scene.add_actor(&actor);
        Cube.el().set(rigid_dynamic(), actor).spawn_static(world);
    }

    elements_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);

    scene
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    App::run_debug_app(|app, runtime| {
        let scene = runtime.block_on(async { init(&mut app.world).await });

        app.systems.add(Box::new(FixedTimestepSystem::new(1. / 60., Box::new(sync_ecs_physics()))));
        app.systems.add(Box::new(FixedTimestepSystem::new(
            1. / 60.,
            Box::new(FnSystem::new(move |_world, _| {
                scene.fetch_results(true);
                scene.simulate(1. / 60.);
            })),
        )));
    });
}
