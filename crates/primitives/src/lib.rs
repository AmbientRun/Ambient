use ambient_core::{
    asset_cache,
    bounding::{local_bounding_aabb, world_bounding_aabb, world_bounding_sphere},
    main_scene, mesh,
    transform::{local_to_world, mesh_to_local, mesh_to_world, rotation, scale, translation},
};
use ambient_ecs::{components, query, Entity, EntityId, Networked, Store, SystemGroup, World};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
pub use ambient_meshes::UVSphereMesh;
use ambient_meshes::{
    CapsuleMesh, CapsuleMeshKey, SphereMeshKey, TorusMesh, TorusMeshKey, UnitCubeMeshKey,
    UnitQuadMeshKey,
};
use ambient_renderer::{
    color, gpu_primitives_lod, gpu_primitives_mesh, material,
    materials::flat_material::{get_flat_shader, FlatMaterialKey},
    primitives, renderer_shader,
};
use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    cb,
    shapes::{Sphere, AABB},
};
use glam::{vec3, Mat4, Quat, Vec3, Vec4};

pub use ambient_ecs::generated::primitives::components::{
    capsule, capsule_half_height, capsule_latitudes, capsule_longitudes, capsule_radius,
    capsule_rings, cube, quad, sphere, sphere_radius, sphere_sectors, sphere_stacks, torus,
    torus_inner_radius, torus_loops, torus_outer_radius, torus_slices,
};

components!("primitives", {
    @[Networked, Store]
    uv_sphere: UVSphereMesh,
});

pub fn cube_data(assets: &AssetCache) -> Entity {
    let aabb = AABB {
        min: -Vec3::ONE * 0.5,
        max: Vec3::ONE * 0.5,
    };
    Entity::new()
        .with(mesh(), UnitCubeMeshKey.get(assets))
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with_default(translation())
        .with(renderer_shader(), cb(get_flat_shader))
        .with(material(), FlatMaterialKey::white().get(assets))
        .with(primitives(), vec![])
        .with_default(gpu_primitives_mesh())
        .with_default(gpu_primitives_lod())
        .with(color(), Vec4::ONE)
        .with(main_scene(), ())
        .with(local_bounding_aabb(), aabb)
        .with(world_bounding_sphere(), aabb.to_sphere())
        .with(world_bounding_aabb(), aabb)
}

pub fn quad_data(assets: &AssetCache) -> Entity {
    let aabb = AABB {
        min: vec3(-0.5, -0.5, 0.),
        max: vec3(0.5, 0.5, 0.),
    };
    Entity::new()
        .with(mesh(), UnitQuadMeshKey.get(assets))
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with_default(translation())
        .with(renderer_shader(), cb(get_flat_shader))
        .with(material(), FlatMaterialKey::white().get(assets))
        .with(primitives(), vec![])
        .with_default(gpu_primitives_mesh())
        .with_default(gpu_primitives_lod())
        .with(color(), Vec4::ONE)
        .with(main_scene(), ())
        .with(local_bounding_aabb(), aabb)
        .with(world_bounding_sphere(), aabb.to_sphere())
        .with(world_bounding_aabb(), aabb)
}

pub fn sphere_data(assets: &AssetCache, sphere: &UVSphereMesh) -> Entity {
    let bound_sphere = Sphere::new(Vec3::ZERO, sphere.radius);
    Entity::new()
        .with(mesh(), SphereMeshKey(*sphere).get(assets))
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with_default(translation())
        .with(renderer_shader(), cb(get_flat_shader))
        .with(material(), FlatMaterialKey::white().get(assets))
        .with(primitives(), vec![])
        .with_default(gpu_primitives_mesh())
        .with_default(gpu_primitives_lod())
        .with(color(), Vec4::ONE)
        .with(main_scene(), ())
        .with(local_bounding_aabb(), bound_sphere.to_aabb())
        .with(world_bounding_aabb(), bound_sphere.to_aabb())
        .with(world_bounding_sphere(), bound_sphere)
}

pub fn torus_data(assets: &AssetCache, torus: &TorusMesh) -> Entity {
    let aabb = AABB {
        min: vec3(
            -torus.outer_radius - torus.inner_radius,
            -torus.outer_radius - torus.inner_radius,
            -torus.inner_radius,
        ),
        max: vec3(
            torus.outer_radius + torus.inner_radius,
            torus.outer_radius + torus.inner_radius,
            torus.inner_radius,
        ),
    };
    Entity::new()
        .with(mesh(), TorusMeshKey(*torus).get(assets))
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with_default(translation())
        .with(renderer_shader(), cb(get_flat_shader))
        .with(material(), FlatMaterialKey::white().get(assets))
        .with(primitives(), vec![])
        .with_default(gpu_primitives_mesh())
        .with_default(gpu_primitives_lod())
        .with(color(), Vec4::ONE)
        .with(main_scene(), ())
        .with(local_bounding_aabb(), aabb)
        .with(world_bounding_aabb(), aabb)
        .with(world_bounding_sphere(), aabb.to_sphere())
}

pub fn capsule_data(assets: &AssetCache, capsule: &CapsuleMesh) -> Entity {
    let aabb = AABB {
        min: vec3(-capsule.radius, -capsule.radius, -capsule.half_height),
        max: vec3(capsule.radius, capsule.radius, capsule.half_height),
    };
    Entity::new()
        .with(mesh(), CapsuleMeshKey(*capsule).get(assets))
        .with_default(local_to_world())
        .with_default(mesh_to_world())
        .with_default(translation())
        .with(renderer_shader(), cb(get_flat_shader))
        .with(material(), FlatMaterialKey::white().get(assets))
        .with(primitives(), vec![])
        .with_default(gpu_primitives_mesh())
        .with_default(gpu_primitives_lod())
        .with(color(), Vec4::ONE)
        .with(main_scene(), ())
        .with(local_bounding_aabb(), aabb)
        .with(world_bounding_aabb(), aabb)
        .with(world_bounding_sphere(), aabb.to_sphere())
}

fn extend(world: &mut World, id: EntityId, data: Entity) {
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
            query((
                sphere_radius().changed(),
                sphere_sectors().changed(),
                sphere_stacks().changed(),
            ))
            .incl(sphere())
            .spawned()
            .to_system(|q, world, qs, _| {
                for (id, (radius, sectors, stacks)) in q.collect_cloned(world, qs) {
                    let mesh = UVSphereMesh {
                        radius,
                        sectors: sectors.try_into().unwrap(),
                        stacks: stacks.try_into().unwrap(),
                    };
                    world.add_component(id, uv_sphere(), mesh).unwrap();
                }
            }),
            query(uv_sphere()).spawned().to_system(|q, world, qs, _| {
                for (id, sphere) in q.collect_cloned(world, qs) {
                    let data = sphere_data(world.resource(asset_cache()), &sphere);
                    extend(world, id, data);
                }
            }),
            query((
                capsule_radius().changed(),
                capsule_half_height().changed(),
                capsule_rings().changed(),
                capsule_latitudes().changed(),
                capsule_longitudes().changed(),
            ))
            .incl(capsule())
            .spawned()
            .to_system(|q, world, qs, _| {
                for (id, (radius, half_height, rings, latitudes, longitudes)) in
                    q.collect_cloned(world, qs)
                {
                    let mesh = CapsuleMesh {
                        radius,
                        half_height,
                        rings: rings.try_into().unwrap(),
                        latitudes: latitudes.try_into().unwrap(),
                        longitudes: longitudes.try_into().unwrap(),
                        ..Default::default()
                    };
                    let data = capsule_data(world.resource(asset_cache()), &mesh);
                    extend(world, id, data);
                }
            }),
            query((
                torus_inner_radius().changed(),
                torus_outer_radius().changed(),
                torus_slices().changed(),
                torus_loops().changed(),
            ))
            .incl(torus())
            .spawned()
            .to_system(|q, world, qs, _| {
                for (id, (inner_radius, outer_radius, loops, slices)) in q.collect_cloned(world, qs)
                {
                    let mesh = TorusMesh {
                        inner_radius,
                        outer_radius,
                        slices,
                        loops,
                    };
                    let data = torus_data(world.resource(asset_cache()), &mesh);
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
            .with(translation(), self.from)
            .with(rotation(), Quat::from_rotation_arc(Vec3::X, d.normalize()))
            .with(scale(), vec3(d.length(), self.thickness, self.thickness))
            .init(
                mesh_to_local(),
                Mat4::from_scale_rotation_translation(
                    Vec3::ONE * 0.5,
                    Quat::IDENTITY,
                    vec3(0.5, 0., 0.),
                ),
            )
    }
}
