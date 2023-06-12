use std::sync::Arc;

use ambient_app::{App, AppBuilder};
use ambient_core::{
    asset_cache, camera::active_camera, gpu, hierarchy::set_component_recursive, main_scene, mesh,
    transform::*,
};
use ambient_ecs::Entity;
use ambient_gpu::{
    gpu::GpuKey,
    shader_module::{BindGroupDesc, ShaderModule},
};
use ambient_meshes::QuadMeshKey;
use ambient_model_import::model_crate::ModelCrate;
use ambient_renderer::{
    gpu_primitives_lod, gpu_primitives_mesh, material,
    materials::flat_material::{get_flat_shader, FlatMaterial},
    primitives, renderer_shader, Material, MaterialShader, SharedMaterial, StandardShaderKey,
    MATERIAL_BIND_GROUP,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    cb, friendly_id,
    math::SphericalCoords,
};
use glam::*;
use wgpu::BindGroup;

fn get_custom_material_layout() -> BindGroupDesc<'static> {
    BindGroupDesc {
        entries: vec![],
        label: MATERIAL_BIND_GROUP.into(),
    }
}

#[derive(Clone, Debug)]
pub struct CustomMaterialShaderKey;
impl SyncAssetKey<Arc<MaterialShader>> for CustomMaterialShaderKey {
    fn load(&self, _assets: AssetCache) -> Arc<MaterialShader> {
        Arc::new(MaterialShader {
            id: "custom_material_shader".to_string(),
            shader: Arc::new(
                ShaderModule::new("CustomMaterial", include_str!("material.wgsl"))
                    .with_binding_desc(get_custom_material_layout()),
            ),
        })
    }
}

pub struct CustomMaterial {
    id: String,
    bind_group: wgpu::BindGroup,
}
impl CustomMaterial {
    pub fn new(assets: &AssetCache) -> Self {
        let gpu = GpuKey.get(assets);
        let layout = get_custom_material_layout().get(assets);

        Self {
            id: friendly_id(),
            bind_group: gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[],
                label: Some("CustomMaterial.bind_group"),
            }),
        }
    }
}
impl std::fmt::Debug for CustomMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomMaterial")
            .field("id", &self.id)
            .finish()
    }
}
impl Material for CustomMaterial {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
    fn id(&self) -> &str {
        &self.id
    }
}

async fn init(app: &mut App) {
    let world = &mut app.world;
    let gpu = world.resource(gpu()).clone();
    let assets = world.resource(asset_cache()).clone();

    let model = ModelCrate::local_import(
        &assets,
        &AbsAssetUrl::parse("assets/Soldier.glb").unwrap(),
        true,
        false,
    )
    .await
    .unwrap();
    let entity = model.spawn(&gpu, world, &Default::default());
    set_component_recursive(
        world,
        entity,
        renderer_shader(),
        cb(|assets, config| {
            StandardShaderKey {
                material_shader: CustomMaterialShaderKey.get(assets),
                lit: true,
                shadow_cascades: config.shadow_cascades,
            }
            .get(assets)
        }),
    );
    set_component_recursive(
        world,
        entity,
        material(),
        SharedMaterial::new(CustomMaterial::new(&assets)),
    );
    // world.add_component(entity, rotation(), glam::Quat::from_rotation_x(std::f32::consts::PI / 2.)).unwrap();
    // world.set(entity, animation_controller(), AnimationController::looping("Walk")).unwrap();

    let grey = FlatMaterial::new(&gpu, &assets, vec4(0.5, 0.5, 0.5, 1.), Some(false));

    Entity::new()
        .with(mesh(), QuadMeshKey.get(&assets))
        .with(renderer_shader(), cb(get_flat_shader))
        .with(material(), SharedMaterial::new(grey))
        .with(primitives(), vec![])
        .with_default(gpu_primitives_mesh())
        .with_default(gpu_primitives_lod())
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with(scale(), vec3(20., 20., 1.))
        .with(main_scene(), ())
        .spawn(world);

    ambient_cameras::spherical::new(
        vec3(0., 0., 0.),
        SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.),
    )
    .with(active_camera(), 0.)
    .with(main_scene(), ())
    .spawn(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple().block_on(init);
}
