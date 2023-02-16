use glam::{vec3, Mat4, Quat, Vec3, Vec4};
use kiwi_core::{
    asset_cache,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    main_scene, mesh,
    transform::{local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation},
};
use kiwi_ecs::{components, query, DefaultValue, Description, EntityData, EntityId, Name, Networked, Store, SystemGroup, World};
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use kiwi_gpu::mesh_buffer::GpuMesh;
pub use kiwi_meshes::UVSphereMesh;
use kiwi_meshes::{UnitCubeMeshKey, UnitQuadMeshKey};
use kiwi_renderer::{
    color, gpu_primitives, material,
    materials::flat_material::{get_flat_shader, FlatMaterialKey},
    primitives, renderer_shader,
};
use kiwi_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    cb,
    mesh::Mesh,
    shapes::{Sphere, AABB},
};

components!("primitives", {
    @[
        Networked, Store,
        Name["Cube"],
        Description["If attached to an entity, the entity will be converted to a cube primitive.\nThe cube is unit-sized (i.e. 0.5 metres out to each side)."]
    ]
    cube: (),
    @[
        Networked, Store,
        Name["Quad"],
        Description["If attached to an entity, the entity will be converted to a quad primitive.\nThe quad is unit-sized on the XY axes, and flat on the Z axis (i.e. 0.5 metres out to the XY axes)."]
    ]
    quad: (),

    @[
        Networked, Store,
        Name["Sphere"],
        Description["If attached to an entity, the entity will be converted to a unit-diameter sphere primitive.\nThe sphere can be customized using the `sphere_radius`, `sphere_sectors` and `sphere_stacks` components."]
    ]
    sphere: (),
    @[
        Networked, Store, DefaultValue<_>[0.5],
        Name["Sphere radius"],
        Description["Set the radius of a `sphere` entity."]
    ]
    sphere_radius: f32,
    @[
        Networked, Store, DefaultValue<_>[36],
        Name["Sphere sectors"],
        Description["Set the longitudinal sectors of a `sphere` entity."]
    ]
    sphere_sectors: u32,
    @[
        Networked, Store, DefaultValue<_>[18],
        Name["Sphere stacks"],
        Description["Set the latitudinal stacks of a `sphere` entity."]
    ]
    sphere_stacks: u32,
    @[Networked, Store]
    uv_sphere: UVSphereMesh,
});

pub fn cube_data(assets: &AssetCache) -> EntityData {
    let aabb = AABB { min: -Vec3::ONE * 0.5, max: Vec3::ONE * 0.5 };
    EntityData::new()
        .set(mesh(), UnitCubeMeshKey.get(assets))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(translation())
        .set(renderer_shader(), cb(get_flat_shader))
        .set(material(), FlatMaterialKey::white().get(assets))
        .set(primitives(), vec![])
        .set_default(gpu_primitives())
        .set(color(), Vec4::ONE)
        .set(main_scene(), ())
        .set(local_bounding_aabb(), aabb)
        .set(world_bounding_sphere(), aabb.to_sphere())
        .set(world_bounding_aabb(), aabb)
}

pub fn quad_data(assets: &AssetCache) -> EntityData {
    let aabb = AABB { min: vec3(-0.5, -0.5, 0.), max: vec3(0.5, 0.5, 0.) };
    EntityData::new()
        .set(mesh(), UnitQuadMeshKey.get(assets))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(translation())
        .set(renderer_shader(), cb(get_flat_shader))
        .set(material(), FlatMaterialKey::white().get(assets))
        .set(primitives(), vec![])
        .set_default(gpu_primitives())
        .set(color(), Vec4::ONE)
        .set(main_scene(), ())
        .set(local_bounding_aabb(), aabb)
        .set(world_bounding_sphere(), aabb.to_sphere())
        .set(world_bounding_aabb(), aabb)
}

pub fn sphere_data(assets: &AssetCache, sphere: &UVSphereMesh) -> EntityData {
    let bound_sphere = Sphere::new(Vec3::ZERO, sphere.radius);
    EntityData::new()
        .set(mesh(), GpuMesh::from_mesh(assets.clone(), &Mesh::from(*sphere)))
        .set_default(local_to_world())
        .set_default(mesh_to_world())
        .set_default(translation())
        .set(renderer_shader(), cb(get_flat_shader))
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
            query((sphere_radius().changed(), sphere_sectors().changed(), sphere_stacks().changed())).incl(sphere()).spawned().to_system(
                |q, world, qs, _| {
                    for (id, (radius, sectors, stacks)) in q.collect_cloned(world, qs) {
                        let mesh = UVSphereMesh { radius, sectors: sectors.try_into().unwrap(), stacks: stacks.try_into().unwrap() };
                        world.add_component(id, uv_sphere(), mesh).unwrap();
                    }
                },
            ),
            query(uv_sphere()).spawned().to_system(|q, world, qs, _| {
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
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        Element::new().init_extend(cube_data(hooks.world.resource(asset_cache())))
    }
}

#[derive(Debug, Clone)]
pub struct Quad;
impl ElementComponent for Quad {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        Element::new().init_extend(quad_data(hooks.world.resource(asset_cache())))
    }
}

#[derive(Debug, Clone, Default)]
pub struct UVSphere {
    pub sphere: UVSphereMesh,
}
impl ElementComponent for UVSphere {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let UVSphere { sphere } = *self;
        Element::new().init_extend(sphere_data(hooks.world.resource(asset_cache()), &sphere))
    }
}

#[derive(Debug, Clone)]
pub struct BoxLine {
    pub from: Vec3,
    pub to: Vec3,
    pub thickness: f32,
}
impl ElementComponent for BoxLine {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let d = self.to - self.from;
        Cube.el()
            .set(translation(), self.from)
            .set(rotation(), Quat::from_rotation_arc(Vec3::X, d.normalize()))
            .set(scale(), vec3(d.length(), self.thickness, self.thickness))
            .init(mesh_to_local(), Mat4::from_scale_rotation_translation(Vec3::ONE * 0.5, Quat::IDENTITY, vec3(0.5, 0., 0.)))
    }
}
