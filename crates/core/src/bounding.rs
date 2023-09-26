use std::{collections::HashSet, sync::Arc};

use ambient_ecs::{
    components,
    generated::rendering::components::{
        local_bounding_aabb_max, local_bounding_aabb_min, world_bounding_aabb_max,
        world_bounding_aabb_min, world_bounding_sphere_center, world_bounding_sphere_radius,
    },
    query, query_mut, Debuggable, EntityId, FramedEventsReader, MaybeResource, Networked, Store,
    System, SystemGroup, World,
};
use ambient_native_std::{
    shapes::{Sphere, AABB},
    sparse_vec::SparseVec,
};
use glam::{uvec4, UVec4};
use itertools::Itertools;

use crate::{gpu, hierarchy::children, transform::local_to_world};
use ambient_gpu::gpu::Gpu;
use ambient_gpu_ecs::gpu_components;
use ambient_gpu_ecs::{
    gpu_world, ArchChangeDetection, ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent,
};

components!("rendering", {
    @[Debuggable, Networked, Store]
    visibility_from: EntityId,
    @[Debuggable, Networked, Store, MaybeResource]
    local_bounding_aabb: AABB,
    @[Debuggable, Networked, Store]
    world_bounding_aabb: AABB,
    @[Debuggable]
    world_bounding_sphere: Sphere,
});
gpu_components! {
    world_bounding_sphere() => world_bounding_sphere: GpuComponentFormat::Vec4,
    visibility_from() => visibility_from: GpuComponentFormat::Vec4,
}

pub fn bounding_systems() -> SystemGroup {
    SystemGroup::new(
        "bounding",
        vec![
            query_mut(
                (world_bounding_aabb(), world_bounding_sphere()),
                (local_bounding_aabb().changed(), local_to_world().changed()),
            )
            .to_system(|q, world, qs, _| {
                for (_, (world_aabb, world_sphere), (aabb, local_to_world)) in q.iter(world, qs) {
                    let world_box = aabb.transform(local_to_world);
                    *world_aabb = world_box.to_aabb();
                    *world_sphere = world_box.to_sphere();
                }
            }),
            // Systems for syncing the structured components to the piecewise components, so
            // that the guest can access them
            Box::new(SystemGroup::new(
                "piecewise_sync",
                vec![
                    query(local_bounding_aabb().changed()).to_system(|q, world, qs, _| {
                        for (id, aabb) in q.collect_cloned(world, qs) {
                            world
                                .add_component(id, local_bounding_aabb_min(), aabb.min)
                                .ok();
                            world
                                .add_component(id, local_bounding_aabb_max(), aabb.max)
                                .ok();
                        }
                    }),
                    query(world_bounding_aabb().changed()).to_system(|q, world, qs, _| {
                        for (id, aabb) in q.collect_cloned(world, qs) {
                            world
                                .add_component(id, world_bounding_aabb_min(), aabb.min)
                                .ok();
                            world
                                .add_component(id, world_bounding_aabb_max(), aabb.max)
                                .ok();
                        }
                    }),
                    query(world_bounding_sphere().changed()).to_system(|q, world, qs, _| {
                        for (id, sphere) in q.collect_cloned(world, qs) {
                            world
                                .add_component(id, world_bounding_sphere_center(), sphere.center)
                                .ok();
                            world
                                .add_component(id, world_bounding_sphere_radius(), sphere.radius)
                                .ok();
                        }
                    }),
                ],
            )),
        ],
    )
}

pub fn gpu_world_systems(gpu: Arc<Gpu>) -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "bounding/gpu_world",
        vec![
            Box::new(ComponentToGpuSystem::new(
                gpu,
                GpuComponentFormat::Vec4,
                world_bounding_sphere(),
                gpu_components::world_bounding_sphere(),
            )),
            Box::new(VisibilityFromToGpuSystem::new()),
        ],
    )
}

pub fn calc_world_bounding_recursive(world: &World, id: EntityId) -> Option<AABB> {
    let mut aabbs = Vec::new();
    if let Ok(aabb) = world.get(id, world_bounding_aabb()) {
        aabbs.push(aabb);
    }
    if let Ok(childs) = world.get_ref(id, children()) {
        for c in childs {
            if let Some(aabb) = calc_world_bounding_recursive(world, *c) {
                aabbs.push(aabb);
            }
        }
    }

    AABB::unions(&aabbs)
}

struct VisibilityFromToGpuSystem {
    entity_sets: SparseVec<HashSet<EntityId>>,
    event_readers: SparseVec<FramedEventsReader<EntityId>>,
    changed: ArchChangeDetection,
}
impl VisibilityFromToGpuSystem {
    fn new() -> Self {
        Self {
            entity_sets: SparseVec::new(),
            event_readers: SparseVec::new(),
            changed: ArchChangeDetection::new(),
        }
    }
}
impl System<GpuWorldSyncEvent> for VisibilityFromToGpuSystem {
    fn run(&mut self, world: &mut World, _: &GpuWorldSyncEvent) {
        profiling::scope!("VisibilityFromToGpu.run");
        let gpu_world = world.resource(gpu_world()).lock();
        let gpu = world.resource(gpu());
        for arch in world.archetypes() {
            if let Some((gpu_buff, offset, layout_version)) = gpu_world.get_buffer(
                GpuComponentFormat::Vec4,
                gpu_components::visibility_from(),
                arch.id,
            ) {
                let content_changed = self
                    .changed
                    .changed(arch, visibility_from(), layout_version);
                let buf = arch.get_component_buffer(visibility_from()).unwrap();
                if content_changed {
                    let entity_set: HashSet<EntityId> = buf.data.iter().copied().collect();
                    self.entity_sets.set(arch.id, entity_set);
                }
                let mut loc_changed = false;
                let reader = self
                    .event_readers
                    .get_mut_or_insert_with(arch.id, FramedEventsReader::new);
                let entity_set = self.entity_sets.get(arch.id).unwrap();
                for (_, id) in reader.iter(world.loc_changed()) {
                    if entity_set.contains(id) {
                        loc_changed = true;
                        break;
                    }
                }
                if content_changed || loc_changed {
                    let data = buf
                        .data
                        .iter()
                        .map(&|value: &EntityId| {
                            if let Some(loc) = world.entity_loc(*value) {
                                uvec4(loc.archetype as u32, loc.index as u32, 0, 0)
                            } else {
                                UVec4::ZERO
                            }
                        })
                        .collect_vec();

                    gpu.queue
                        .write_buffer(gpu_buff, offset, bytemuck::cast_slice(&data));
                }
            }
        }
    }
}
impl std::fmt::Debug for VisibilityFromToGpuSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VisibilityFromToGpu").finish()
    }
}
