use ambient_animation::{animation_controller, AnimationController};
use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache,
    camera::{active_camera, far},
    main_scene,
    transform::*,
};
use ambient_ecs::FnSystem;
use ambient_element::ElementComponentExt;
use ambient_model_import::model_crate::ModelCrate;
use ambient_primitives::{Cube, Quad};
use ambient_renderer::color;
use ambient_std::{
    asset_url::{AbsAssetUrl, TypedAssetUrl},
    math::SphericalCoords,
};
use glam::*;
use winit::event::{Event, VirtualKeyCode, WindowEvent};

async fn init(app: &mut App) {
    let world = &mut app.world;

    let assets = world.resource(asset_cache()).clone();

    Cube.el().set(translation(), vec3(8., 0., 0.)).set(color(), vec4(1., 0., 0., 1.)).spawn_static(world);
    Cube.el().set(translation(), vec3(0., 8., 0.)).set(color(), vec4(0., 1., 0., 1.)).spawn_static(world);
    Quad.el().set(scale(), Vec3::ONE * 10.).spawn_static(world);

    let model = ModelCrate::local_import(
        &assets,
        &AbsAssetUrl::parse("https://playdims.com/api/v1/assetdb/packs/OzYr8KHH0OKtFamZLB8U/content/Vanguard By T. Choonyung.fbx").unwrap(),
        true,
        false,
    )
    .await
    .unwrap();

    let entities = model.batch_spawn(world, &Default::default(), 2);
    world.add_component(entities[1], translation(), vec3(2., 0., 0.)).unwrap();

    world
        .add_component(
            entities[0],
            animation_controller(),
            AnimationController::looping(TypedAssetUrl::parse("https://playdims.com/api/v1/assetdb/crates/Y3FxdwvLWHBtJMsQTNbt/1.0.0/Y3FxdwvLWHBtJMsQTNbt-bae68692bdaf9a5fcef953945904bd74f2b21fd1d8cadc789b410ec8a559f0f6/animations/mixamo.com.anim").unwrap()),
        )
        .unwrap();

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .set(far(), 2000.)
        .spawn(world);

    let entity = entities[0];

    app.window_event_systems.add(Box::new(FnSystem::new(move |world, event| {
            if let Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } = event {
                if let Some(keycode) = input.virtual_keycode {
                    match keycode {
                        // VirtualKeyCode::Z => add_bone_visualization(world, entity),
                        VirtualKeyCode::Key1 => {
                            world
                                .set(
                                    entity,
                                    animation_controller(),
                                    AnimationController::looping(TypedAssetUrl::parse("https://playdims.com/api/v1/assetdb/crates/Y3FxdwvLWHBtJMsQTNbt/1.0.0/Y3FxdwvLWHBtJMsQTNbt-0104f92c03b53349f19f365cfaccd0a92a18a4d13c7c262fc52b84347eda4fe0/animations/mixamo.com.anim").unwrap()),
                                )
                                .unwrap();
                        }
                        VirtualKeyCode::Key2 => {
                            world
                                .set(
                                    entity,
                                    animation_controller(),
                                    AnimationController::looping(TypedAssetUrl::parse("https://playdims.com/api/v1/assetdb/crates/Y3FxdwvLWHBtJMsQTNbt/1.0.0/Y3FxdwvLWHBtJMsQTNbt-bae68692bdaf9a5fcef953945904bd74f2b21fd1d8cadc789b410ec8a559f0f6/animations/mixamo.com.anim").unwrap()),
                                )
                                .unwrap();
                        }
                        VirtualKeyCode::Key3 => {
                            world
                                .set(
                                    entity,
                                    animation_controller(),
                                    AnimationController::looping(TypedAssetUrl::parse("https://playdims.com/api/v1/assetdb/crates/Y3FxdwvLWHBtJMsQTNbt/1.0.0/Y3FxdwvLWHBtJMsQTNbt-70b97a74b6d43492295ae55bd621b51d04d8e86c5e73e172f68a82e9b56daaa0/animations/mixamo.com.anim").unwrap()),
                                )
                                .unwrap();
                        }
                        _ => {}
                    }
                }
            }
        })));
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    AppBuilder::simple().block_on(init);
}
