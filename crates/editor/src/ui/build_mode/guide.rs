use glam::{vec2, vec3, vec4, EulerRot, Mat4, Quat, Vec2, Vec3};
use kiwi_core::{
    asset_cache,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    main_scene, mesh,
    transform::{local_to_world, mesh_to_world},
};
use kiwi_ecs::{EntityData, EntityId, World};
use kiwi_element::{Element, ElementComponent};
use kiwi_meshes::QuadMeshKey;
use kiwi_network::client::GameClient;
use kiwi_renderer::{color, double_sided, gpu_primitives, material, primitives, renderer_shader, SharedMaterial, StandardShaderKey};
use kiwi_std::{asset_cache::SyncAssetKeyExt, cb, shapes::AABB};

use super::grid_material::{GridMaterialKey, GridShaderKey};
use crate::GRID_SIZE;

const BLUEBOARD_SIZE: f32 = 1024.0;
const LINE_WIDTH: f32 = 0.1;

fn spawn_entity(world: &mut World, mat: SharedMaterial) -> EntityId {
    let assets = world.resource(asset_cache());

    let aabb = AABB { min: vec3(-1., -1., 0.), max: vec3(1., 1., 0.) };

    EntityData::new()
        .set(mesh(), QuadMeshKey.get(assets))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set(primitives(), vec![])
        .set_default(gpu_primitives())
        .set(main_scene(), ())
        .set(local_bounding_aabb(), aabb)
        .set(world_bounding_sphere(), aabb.to_sphere())
        .set(world_bounding_aabb(), aabb)
        .set(color(), vec4(0.3, 0.3, 1., 1.0))
        .set(double_sided(), true)
        .set(material(), mat)
        .set(
            renderer_shader(),
            cb(|assets, config| {
                StandardShaderKey { material_shader: GridShaderKey.get(assets), lit: false, shadow_cascades: config.shadow_cascades }
                    .get(assets)
            }),
        )
        .spawn(world)
}

#[derive(Debug, Clone)]
pub struct GridGuide {
    pub rotation: Quat,
    pub point: Vec3,
}

impl ElementComponent for GridGuide {
    fn render(self: Box<Self>, hooks: &mut kiwi_element::Hooks) -> kiwi_element::Element {
        let Self { rotation, point } = *self;

        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();

        let (entity, _) = hooks.use_state_with(|world| {
            let assets = world.resource(asset_cache());

            let mut state = game_client.game_state.lock();
            let mat = GridMaterialKey {
                major: Vec2::splat(1.0 / (GRID_SIZE * 5.0)),
                minor: Vec2::splat(1.0 / GRID_SIZE),
                line_width: 0.2,
                size: BLUEBOARD_SIZE,
            }
            .get(assets);

            spawn_entity(&mut state.world, mat)
        });

        {
            let game_state = game_client.game_state.clone();
            hooks.use_spawn(move |_| {
                Box::new(move |_| {
                    game_state.lock().world.despawn(entity);
                })
            });
        }

        hooks.use_effect((rotation, point), |_, &(rotation, point)| {
            let mut state = game_client.game_state.lock();
            let _euler = rotation.to_euler(EulerRot::YXZ);

            let transform = Mat4::from_scale_rotation_translation(Vec3::splat(BLUEBOARD_SIZE), rotation, point);
            state.world.set(entity, local_to_world(), transform).expect("Entity was despawned");

            Box::new(|_| {})
        });

        Element::new()
    }
}

#[derive(Debug, Clone)]
pub struct AxisGuide {
    pub axis: Vec3,
    pub point: Vec3,
}

impl ElementComponent for AxisGuide {
    #[tracing::instrument(skip_all)]
    fn render(self: Box<Self>, hooks: &mut kiwi_element::Hooks) -> kiwi_element::Element {
        let Self { axis, point } = *self;

        let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();

        let (entity, _) = hooks.use_state_with(|world| {
            let mut state = game_client.game_state.lock();
            let assets = world.resource(asset_cache());

            let mat = GridMaterialKey { major: vec2(0.0, 0.2), minor: vec2(0.0, 2.0), line_width: 0.2, size: BLUEBOARD_SIZE }.get(assets);

            spawn_entity(&mut state.world, mat)
        });

        {
            let game_state = game_client.game_state.clone();
            hooks.use_spawn(move |_| {
                Box::new(move |_| {
                    game_state.lock().world.despawn(entity);
                })
            });
        }

        let mut state = game_client.game_state.lock();

        assert!(axis.is_normalized(), "axis: {axis}");
        let view = state.view().unwrap_or_default();

        let camera_pos = view.inverse().transform_point3(Vec3::ZERO);

        let to_camera = point - camera_pos;

        // Rotate the plane to face the axis
        let rot = Quat::from_rotation_arc(Vec3::Y, axis);

        let tangent = rot * Vec3::Z;

        // Flatten along the axis
        let to_camera = to_camera.reject_from(axis).normalize_or_zero();

        let billboard = Quat::from_rotation_arc(tangent, to_camera);

        let transform = Mat4::from_scale_rotation_translation(vec3(LINE_WIDTH, BLUEBOARD_SIZE, BLUEBOARD_SIZE), billboard * rot, point);

        state.world.set(entity, local_to_world(), transform).expect("Entity was despawned");

        Element::new()
    }
}
