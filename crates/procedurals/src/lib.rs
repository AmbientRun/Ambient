use std::{collections::HashMap, sync::Arc};

use ambient_core::{
    asset_cache,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    main_scene,
    transform::{local_to_world, mesh_to_world},
};
use ambient_ecs::{
    components,
    generated::components::core::procedurals::{procedural_material, procedural_mesh},
    query, Entity, Resource, SystemGroup,
};
use ambient_gpu::{mesh_buffer::GpuMesh, texture::TextureView};
use ambient_renderer::{
    gpu_primitives_lod, gpu_primitives_mesh,
    pbr_material::{get_pbr_shader, PbrMaterial, PbrMaterialConfig},
    primitives, renderer_shader, SharedMaterial,
};
use ambient_shared_types::{
    ProceduralMaterialHandle, ProceduralMeshHandle, ProceduralSamplerHandle,
    ProceduralTextureHandle,
};
use ambient_std::{cb, mesh::Mesh};

components!("procedurals", {
    @[Resource]
    procedural_storage: ProceduralStorage,
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "procedurals",
        vec![
            query(procedural_mesh().changed()).to_system(|query, world, query_state, _| {
                let assets = world.resource(asset_cache()).clone();
                for (id, mesh_handle) in query.collect_cloned(world, query_state) {
                    let (gpu_mesh, mesh_aabb) = {
                        let storage = world.resource(procedural_storage());
                        let mesh = storage.get_mesh(mesh_handle);
                        let mesh_aabb = mesh.aabb();
                        let gpu_mesh = GpuMesh::from_mesh(&assets, &mesh);
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
                for (id, material_handle) in query.collect_cloned(world, query_state) {
                    let storage = world.resource(procedural_storage());
                    let material = storage.get_material(material_handle).clone();
                    let material = PbrMaterial::new(&assets, material);
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

pub type ProceduralMesh = Mesh;
pub type ProceduralTexture = Arc<TextureView>;
pub type ProceduralSampler = Arc<wgpu::Sampler>;
pub type ProceduralMaterial = PbrMaterialConfig;

#[derive(Clone)]
pub struct ProceduralStorage {
    meshes: HashMap<ProceduralMeshHandle, ProceduralMesh>,
    textures: HashMap<ProceduralTextureHandle, ProceduralTexture>,
    samplers: HashMap<ProceduralSamplerHandle, ProceduralSampler>,
    materials: HashMap<ProceduralMaterialHandle, ProceduralMaterial>,
}

impl ProceduralStorage {
    pub fn new() -> Self {
        Self {
            meshes: Default::default(),
            textures: Default::default(),
            samplers: Default::default(),
            materials: Default::default(),
        }
    }

    pub fn insert_mesh(&mut self, mesh: ProceduralMesh) -> ProceduralMeshHandle {
        let handle = ProceduralMeshHandle::new();
        self.meshes.insert(handle, mesh);
        handle
    }

    pub fn insert_texture(&mut self, texture: ProceduralTexture) -> ProceduralTextureHandle {
        let handle = ProceduralTextureHandle::new();
        self.textures.insert(handle, texture);
        handle
    }

    pub fn insert_sampler(&mut self, sampler: ProceduralSampler) -> ProceduralSamplerHandle {
        let handle = ProceduralSamplerHandle::new();
        self.samplers.insert(handle, sampler);
        handle
    }

    pub fn insert_material(&mut self, material: ProceduralMaterial) -> ProceduralMaterialHandle {
        let handle = ProceduralMaterialHandle::new();
        self.materials.insert(handle, material);
        handle
    }

    pub fn remove_mesh(&mut self, handle: ProceduralMeshHandle) -> ProceduralMesh {
        self.meshes
            .remove(&handle)
            .unwrap_or_else(|| panic!("Procedural mesh {handle} must exist"))
    }

    pub fn remove_texture(&mut self, handle: ProceduralTextureHandle) -> ProceduralTexture {
        self.textures
            .remove(&handle)
            .unwrap_or_else(|| panic!("Procedural texture {handle} must exist"))
    }

    pub fn remove_sampler(&mut self, handle: ProceduralSamplerHandle) -> ProceduralSampler {
        self.samplers
            .remove(&handle)
            .unwrap_or_else(|| panic!("Procedural sampler {handle} must exist"))
    }

    pub fn remove_material(&mut self, handle: ProceduralMaterialHandle) -> ProceduralMaterial {
        self.materials
            .remove(&handle)
            .unwrap_or_else(|| panic!("Procedural material {handle} must exist"))
    }

    pub fn get_mesh(&self, handle: ProceduralMeshHandle) -> &ProceduralMesh {
        &self
            .meshes
            .get(&handle)
            .unwrap_or_else(|| panic!("Procedural mesh {handle} must exist"))
    }

    pub fn get_texture(&self, handle: ProceduralTextureHandle) -> &ProceduralTexture {
        &self
            .textures
            .get(&handle)
            .unwrap_or_else(|| panic!("Procedural texture {handle} must exist"))
    }

    pub fn get_sampler(&self, handle: ProceduralSamplerHandle) -> &ProceduralSampler {
        &self
            .samplers
            .get(&handle)
            .unwrap_or_else(|| panic!("Procedural sampler {handle} must exist"))
    }

    pub fn get_material(&self, handle: ProceduralMaterialHandle) -> &ProceduralMaterial {
        &self
            .materials
            .get(&handle)
            .unwrap_or_else(|| panic!("Procedural material {handle} must exist"))
    }
}
