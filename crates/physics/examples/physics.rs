use ambient_app::{gpu, AppBuilder};
use ambient_core::{asset_cache, camera::active_camera, main_scene, transform::scale, FixedTimestepSystem};
use ambient_ecs::{FnSystem, World};
use ambient_element::ElementComponentExt;
use ambient_physics::physx::{physics_controlled, rigid_dynamic, rigid_static, sync_ecs_physics, PhysicsKey};
use ambient_primitives::{Cube, Quad};
use ambient_renderer::color;
use ambient_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};
use glam::*;
use physxx::*;
use rand::random;

fn init(world: &mut World) -> PxSceneRef {
    let _gpu = world.resource(gpu()).clone();
    let assets = world.resource(asset_cache()).clone();
    let physics = PhysicsKey.get(&assets);
    world.add_resource(ambient_physics::physx::physics(), physics.clone());

    let scene = {
        let mut scene_desc = PxSceneDesc::new(physics.physics);
        scene_desc.set_cpu_dispatcher(&physics.dispatcher);
        scene_desc.set_gravity(vec3(0., 0., -9.82));
        PxSceneRef::new(&physics.physics, &scene_desc)
    };

    // Ground plane
    let physics_material = PxMaterial::new(physics.physics, 0.5, 0.5, 0.6);
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
        Cube.el().set(rigid_dynamic(), actor).set_default(physics_controlled()).spawn_static(world);
    }

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);

    scene
}

#[tokio::main]
async fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    AppBuilder::simple()
        .run(|app, _| {
            ambient_physics::init_all_components();
            let scene = init(&mut app.world);

            app.systems.add(Box::new(FixedTimestepSystem::new(1. / 60., Box::new(sync_ecs_physics()))));
            app.systems.add(Box::new(FixedTimestepSystem::new(
                1. / 60.,
                Box::new(FnSystem::new(move |_world, _| {
                    scene.fetch_results(true);
                    scene.simulate(1. / 60.);
                })),
            )));
        })
        .await
}
