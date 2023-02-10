use glam::*;
use kiwi_app::{gpu, AppBuilder};
use kiwi_core::{asset_cache, camera::active_camera, main_scene, transform::*};
use kiwi_ecs::{EntityData, EntityId, FnSystem, World};
use kiwi_meshes::{CubeMeshKey, QuadMeshKey};
use kiwi_renderer::{
    gpu_primitives, materials::flat_material::{get_flat_shader, FlatMaterial}, primitives, RenderPrimitive, SharedMaterial
};
use kiwi_std::{asset_cache::SyncAssetKeyExt, math::SphericalCoords};
use winit::event::{Event, VirtualKeyCode, WindowEvent};

fn init(world: &mut World) -> (EntityId, EntityId, SharedMaterial, SharedMaterial) {
    let _gpu = world.resource(gpu()).clone();
    let assets = world.resource(asset_cache()).clone();
    let flat_static_shader = get_flat_shader(&assets);

    let red = SharedMaterial::new(FlatMaterial::new(assets.clone(), vec4(1., 0., 0., 1.), Some(false)));
    let green = SharedMaterial::new(FlatMaterial::new(assets.clone(), vec4(0., 1., 0., 1.), Some(false)));

    let entity1 = EntityData::new()
        .set(
            primitives(),
            vec![RenderPrimitive { shader: flat_static_shader.clone(), material: red.clone(), mesh: CubeMeshKey.get(&assets), lod: 0 }],
        )
        .set_default(gpu_primitives())
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set(translation(), vec3(-2.5, 0., 0.))
        .set(main_scene(), ())
        .spawn(world);

    let entity2 = EntityData::new()
        .set(
            primitives(),
            vec![RenderPrimitive { shader: flat_static_shader, material: green.clone(), mesh: CubeMeshKey.get(&assets), lod: 0 }],
        )
        .set_default(gpu_primitives())
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set(translation(), vec3(2.5, 0., 0.))
        .set(main_scene(), ())
        .spawn(world);

    kiwi_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(main_scene(), ())
        .spawn(world);
    (entity1, entity2, red, green)
}

fn main() {
    env_logger::init();
    AppBuilder::simple().run(|app, _| {
        let assets = app.world.resource(asset_cache()).clone();
        let (entity1, entity2, material1, material2) = init(&mut app.world);
        app.window_event_systems.add(Box::new(FnSystem::new(move |world, event| {
            if let Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } = event {
                if let Some(keycode) = input.virtual_keycode {
                    match keycode {
                        VirtualKeyCode::Key1 => {
                            world.get_mut(entity1, primitives()).unwrap()[0].material = material1.clone();
                        }
                        VirtualKeyCode::Key2 => {
                            world.get_mut(entity1, primitives()).unwrap()[0].material = material2.clone();
                        }
                        VirtualKeyCode::Key3 => {
                            world.get_mut(entity1, primitives()).unwrap()[0].mesh = CubeMeshKey.get(&assets);
                        }
                        VirtualKeyCode::Key4 => {
                            world.get_mut(entity1, primitives()).unwrap()[0].mesh = QuadMeshKey.get(&assets);
                        }
                        VirtualKeyCode::Key5 => {
                            world.get_mut(entity2, primitives()).unwrap()[0].material = material1.clone();
                        }
                        VirtualKeyCode::Key6 => {
                            world.get_mut(entity2, primitives()).unwrap()[0].material = material2.clone();
                        }
                        VirtualKeyCode::Key7 => {
                            world.get_mut(entity2, primitives()).unwrap()[0].mesh = CubeMeshKey.get(&assets);
                        }
                        VirtualKeyCode::Key8 => {
                            world.get_mut(entity2, primitives()).unwrap()[0].mesh = QuadMeshKey.get(&assets);
                        }
                        VirtualKeyCode::Key9 => {
                            material1.borrow_downcast::<FlatMaterial>().update_color(vec4(1., 0., 0., 1.));
                        }
                        VirtualKeyCode::Key0 => {
                            material1.borrow_downcast::<FlatMaterial>().update_color(vec4(0., 0., 1., 1.));
                        }
                        _ => {}
                    }
                }
            }
        })));
    });
}
