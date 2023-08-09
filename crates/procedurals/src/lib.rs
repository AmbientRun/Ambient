use std::{collections::HashMap, sync::Arc};

use ambient_core::{
    asset_cache,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    gpu, main_scene,
    transform::{local_to_world, mesh_to_world},
};
use ambient_ecs::{
    components,
    generated::components::core::procedurals::{procedural_material, procedural_mesh},
    query, Entity, Resource, SystemGroup,
};
use ambient_gpu::{mesh_buffer::GpuMesh, texture::TextureView};
use ambient_native_std::{cb, mesh::Mesh};
use ambient_renderer::{
    gpu_primitives_lod, gpu_primitives_mesh,
    pbr_material::{get_pbr_shader, PbrMaterial, PbrMaterialConfig},
    primitives, renderer_shader, SharedMaterial,
};
use ambient_shared_types::{
    procedural_storage_handle_definitions, ProceduralMaterialHandle, ProceduralMeshHandle,
    ProceduralSamplerHandle, ProceduralTextureHandle,
};
use paste::paste;

components!("procedurals", {
    @[Resource]
    procedural_storage: ProceduralStorage,
});

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "procedurals",
        vec![
            query(procedural_mesh().changed()).to_system(|query, world, query_state, _| {
                let assets = world.resource(asset_cache()).clone();
                let gpu = world.resource(gpu()).clone();
                for (id, mesh_handle) in query.collect_cloned(world, query_state) {
                    let (gpu_mesh, mesh_aabb) = {
                        let storage = world.resource(procedural_storage());
                        let mesh = storage.meshes.get(mesh_handle);
                        let mesh_aabb = mesh.aabb();
                        let gpu_mesh = GpuMesh::from_mesh(&gpu, &assets, mesh);
                        (gpu_mesh, mesh_aabb)
                    };
                    world
                        .add_components(
                            id,
                            Entity::new()
                                .with(ambient_core::mesh(), gpu_mesh)
                                .with_default(main_scene())
                                .with_default(gpu_primitives_mesh())
                                .with_default(gpu_primitives_lod())
                                .with_default(primitives())
                                .with_default(local_to_world())
                                .with_default(mesh_to_world())
                                .with(local_bounding_aabb(), mesh_aabb)
                                .with(world_bounding_aabb(), mesh_aabb)
                                .with(world_bounding_sphere(), mesh_aabb.to_sphere()),
                        )
                        .unwrap();
                }
            }),
            query(procedural_material().changed()).to_system(|query, world, query_state, _| {
                let assets = world.resource(asset_cache()).clone();
                let gpu = world.resource(gpu()).clone();
                for (id, material_handle) in query.collect_cloned(world, query_state) {
                    let storage = world.resource(procedural_storage());
                    let material = storage.materials.get(material_handle).clone();
                    let material = PbrMaterial::new(&gpu, &assets, material);
                    let material = SharedMaterial::new(material);
                    world
                        .add_components(
                            id,
                            Entity::new()
                                .with(ambient_renderer::material(), material)
                                .with(renderer_shader(), cb(get_pbr_shader)),
                        )
                        .unwrap();
                }
            }),
        ],
    )
}

macro_rules! make_procedural_storage_new_fns {
    ($($name:ident),*) => { paste!{$(
        #[must_use]
        pub fn [<new_ $name _handle>]() -> [<Procedural $name:camel Handle>] {
            ambient_native_std::ulid().into()
        }
    )*}};
}

procedural_storage_handle_definitions!(make_procedural_storage_new_fns);

pub type ProceduralMesh = Mesh;
pub type ProceduralTexture = Arc<TextureView>;
pub type ProceduralSampler = Arc<wgpu::Sampler>;
pub type ProceduralMaterial = PbrMaterialConfig;

#[derive(Clone)]
pub struct ProceduralMap<Handle, Resource>(HashMap<Handle, Resource>);

impl<Handle, Resource> ProceduralMap<Handle, Resource>
where
    Handle: Eq + std::hash::Hash + std::fmt::Display,
{
    pub fn insert(&mut self, handle: Handle, resource: Resource) {
        self.0.insert(handle, resource);
    }

    pub fn get(&self, handle: Handle) -> &Resource {
        self.0
            .get(&handle)
            .unwrap_or_else(|| panic!("Procedural resource {handle} must exist"))
    }

    pub fn remove(&mut self, handle: Handle) -> Resource {
        self.0
            .remove(&handle)
            .unwrap_or_else(|| panic!("Procedural resource {handle} must exist"))
    }
}

impl<Handle, Resource> Default for ProceduralMap<Handle, Resource> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Clone)]
pub struct ProceduralStorage {
    pub meshes: ProceduralMap<ProceduralMeshHandle, ProceduralMesh>,
    pub textures: ProceduralMap<ProceduralTextureHandle, ProceduralTexture>,
    pub samplers: ProceduralMap<ProceduralSamplerHandle, ProceduralSampler>,
    pub materials: ProceduralMap<ProceduralMaterialHandle, ProceduralMaterial>,
}

impl ProceduralStorage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            meshes: Default::default(),
            textures: Default::default(),
            samplers: Default::default(),
            materials: Default::default(),
        }
    }
}

impl Default for ProceduralStorage {
    fn default() -> Self {
        Self::new()
    }
}
