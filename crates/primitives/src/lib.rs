use elements_core::{
    asset_cache, bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere}, main_scene, mesh, transform::{local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation}
};
use elements_ecs::{components, query, EntityData, EntityId, Networked, Store, SystemGroup, World};
use elements_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use elements_gpu::mesh_buffer::GpuMesh;
pub use elements_meshes::UVSphereMesh;
use elements_meshes::{CubeMeshKey, QuadMeshKey};
use elements_renderer::{
    color, gpu_primitives, material, materials::flat_material::{get_flat_shader, FlatMaterialKey}, primitives, renderer_shader
};
use elements_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt}, mesh::Mesh, shapes::{Sphere, AABB}
};
use glam::{vec3, Mat4, Quat, Vec3, Vec4};

components!("primitives", {
    @[Networked, Store]
    cube: (),
    @[Networked, Store]
    quad: (),
    @[Networked, Store]
    sphere: UVSphereMesh,
});

fn cube_data(assets: &AssetCache) -> EntityData {
    let aabb = AABB { min: -Vec3::ONE, max: Vec3::ONE };
    EntityData::new()
        .set(mesh(), CubeMeshKey.get(assets))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(translation())
        .set(renderer_shader(), get_flat_shader(assets))
        .set(material(), FlatMaterialKey::white().get(assets))
        .set(primitives(), vec![])
        .set_default(gpu_primitives())
        .set(color(), Vec4::ONE)
        .set(main_scene(), ())
        .set(local_bounding_aabb(), aabb)
        .set(world_bounding_sphere(), aabb.to_sphere())
        .set(world_bounding_aabb(), aabb)
}

fn quad_data(assets: &AssetCache) -> EntityData {
    let aabb = AABB { min: vec3(-1., -1., 0.), max: vec3(1., 1., 0.) };
    EntityData::new()
        .set(mesh(), QuadMeshKey.get(assets))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(translation())
        .set(renderer_shader(), get_flat_shader(assets))
        .set(material(), FlatMaterialKey::white().get(assets))
        .set(primitives(), vec![])
        .set_default(gpu_primitives())
        .set(color(), Vec4::ONE)
        .set(main_scene(), ())
        .set(local_bounding_aabb(), aabb)
        .set(world_bounding_sphere(), aabb.to_sphere())
        .set(world_bounding_aabb(), aabb)
}

fn sphere_data(assets: &AssetCache, sphere: &UVSphereMesh) -> EntityData {
    let bound_sphere = Sphere::new(Vec3::ZERO, sphere.radius);
    EntityData::new()
        .set(mesh(), GpuMesh::from_mesh(assets.clone(), &Mesh::from(*sphere)))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(translation())
        .set(renderer_shader(), get_flat_shader(assets))
        .set(material(), FlatMaterialKey::white().get(assets))
        .set(primitives(), vec![])
        .set_default(gpu_primitives())
        .set(color(), Vec4::ONE)
        .set(main_scene(), ())
        .set(local_bounding_aabb(), bound_sphere.to_aabb())
        .set(world_bounding_aabb(), bound_sphere.to_aabb())
        .set(world_bounding_sphere(), bound_sphere)
}

fn extend(world: &mut World, id: EntityId, data: EntityData) {
    for entry in data {
        if !world.has_component(id, entry.desc()) {
            world.add_entry(id, entry).unwrap();
        }
    }
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "primitives",
        vec![
            query(cube()).spawned().to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    let data = cube_data(world.resource(asset_cache()));
                    extend(world, id, data);
                }
            }),
            query(quad()).spawned().to_system(|q, world, qs, _| {
                for (id, _) in q.collect_cloned(world, qs) {
                    let data = quad_data(world.resource(asset_cache()));
                    extend(world, id, data);
                }
            }),
            query(sphere()).spawned().to_system(|q, world, qs, _| {
                for (id, sphere) in q.collect_cloned(world, qs) {
                    let data = sphere_data(world.resource(asset_cache()), &sphere);
                    extend(world, id, data);
                }
            }),
        ],
    )
}

#[derive(Debug, Clone)]
pub struct Cube;
impl ElementComponent for Cube {
    fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
        Element::new().init_extend(cube_data(world.resource(asset_cache())))
    }
}

#[derive(Debug, Clone)]
pub struct Quad;
impl ElementComponent for Quad {
    fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
        Element::new().init_extend(quad_data(world.resource(asset_cache())))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UVSphere {
    pub sphere: UVSphereMesh,
}
impl ElementComponent for UVSphere {
    fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
        let UVSphere { sphere } = *self;
        Element::new().init_extend(sphere_data(world.resource(asset_cache()), &sphere))
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
