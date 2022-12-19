use elements_core::{
    asset_cache, bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere}, main_scene, mesh, transform::{local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation}
};
use elements_ecs::World;
use elements_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use elements_gpu::mesh_buffer::GpuMesh;
pub use elements_meshes::UVSphereMesh;
use elements_meshes::{CubeMeshKey, QuadMeshKey};
use elements_renderer::{
    color, gpu_primitives, material, materials::flat_material::{get_flat_shader, FlatMaterialKey}, primitives, renderer_shader
};
use elements_std::{
    asset_cache::SyncAssetKeyExt, mesh::Mesh, shapes::{Sphere, AABB}
};
use glam::{vec3, Mat4, Quat, Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct Cube;
impl ElementComponent for Cube {
    fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
        let assets = world.resource(asset_cache());
        let aabb = AABB { min: -Vec3::ONE, max: Vec3::ONE };
        Element::new()
            .init(mesh(), CubeMeshKey.get(assets))
            .init_default(local_to_world())
            .init_default(mesh_to_world())
            .init_default(translation())
            .init(renderer_shader(), get_flat_shader(assets))
            .init(material(), FlatMaterialKey::white().get(assets))
            .init(primitives(), vec![])
            .init_default(gpu_primitives())
            .init(color(), Vec4::ONE)
            .init(main_scene(), ())
            .init(local_bounding_aabb(), aabb)
            .init(world_bounding_sphere(), aabb.to_sphere())
            .init(world_bounding_aabb(), aabb)
    }
}

#[derive(Debug, Clone)]
pub struct Quad;
impl ElementComponent for Quad {
    fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
        let assets = world.resource(asset_cache());
        let aabb = AABB { min: vec3(-1., -1., 0.), max: vec3(1., 1., 0.) };
        Element::new()
            .init(mesh(), QuadMeshKey.get(assets))
            .init_default(local_to_world())
            .init_default(mesh_to_world())
            .init_default(translation())
            .init(renderer_shader(), get_flat_shader(assets))
            .init(material(), FlatMaterialKey::white().get(assets))
            .init(primitives(), vec![])
            .init_default(gpu_primitives())
            .init(color(), Vec4::ONE)
            .init(main_scene(), ())
            .init(local_bounding_aabb(), aabb)
            .init(world_bounding_sphere(), aabb.to_sphere())
            .init(world_bounding_aabb(), aabb)
    }
}

#[derive(Debug, Clone, Default)]
pub struct UVSphere {
    pub sphere: UVSphereMesh,
}
impl ElementComponent for UVSphere {
    fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
        let UVSphere { sphere } = *self;
        let assets = world.resource(asset_cache()).clone();
        let bound_sphere = Sphere::new(Vec3::ZERO, sphere.radius);
        Element::new()
            .init_with(mesh(), move |world| GpuMesh::from_mesh(world.resource(asset_cache()).clone(), &Mesh::from(sphere)))
            .init_default(local_to_world())
            .init_default(mesh_to_world())
            .init_default(translation())
            .init(renderer_shader(), get_flat_shader(&assets))
            .init(material(), FlatMaterialKey::white().get(&assets))
            .init(primitives(), vec![])
            .init_default(gpu_primitives())
            .init(color(), Vec4::ONE)
            .init(main_scene(), ())
            .init(local_bounding_aabb(), bound_sphere.to_aabb())
            .init(world_bounding_aabb(), bound_sphere.to_aabb())
            .init(world_bounding_sphere(), bound_sphere)
    }
}

#[derive(Debug, Clone)]
pub struct BoxLine {
    pub from: Vec3,
    pub to: Vec3,
    pub thickness: f32,
}
impl ElementComponent for BoxLine {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let d = self.to - self.from;
        Cube.el()
            .set(translation(), self.from)
            .set(rotation(), Quat::from_rotation_arc(Vec3::X, d.normalize()))
            .set(scale(), vec3(d.length(), self.thickness, self.thickness))
            .init(mesh_to_local(), Mat4::from_scale_rotation_translation(Vec3::ONE * 0.5, Quat::IDENTITY, vec3(0.5, 0., 0.)))
    }
}
