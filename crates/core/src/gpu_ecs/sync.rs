use std::fmt::Debug;

use ambient_ecs::{
    Archetype, ArchetypeFilter, Component, ComponentDesc, ComponentValue, EntityId, System, World,
};
use ambient_native_std::sparse_vec::SparseVec;
use itertools::Itertools;

use super::{gpu_world, GpuComponentFormat, GpuComponentId};
use crate::gpu;

/// GpuWorld sync/update systems need to run immediately after the GpuWorld has updated it's layout,
/// so by forcing them to use this even we can make sure they're all in order
pub struct GpuWorldSyncEvent;

pub struct ArchChangeDetection {
    arch_data_versions: SparseVec<u64>,
    arch_layout_versions: SparseVec<u64>,
}
impl ArchChangeDetection {
    pub fn new() -> Self {
        Self {
            arch_data_versions: SparseVec::new(),
            arch_layout_versions: SparseVec::new(),
        }
    }
    pub fn changed(
        &mut self,
        arch: &Archetype,
        component: impl Into<ComponentDesc>,
        layout_version: u64,
    ) -> bool {
        let prev_data_version = self.arch_data_versions.get(arch.id).copied();
        let prev_layout_version = self.arch_layout_versions.get(arch.id).copied();
        let data_version = arch.get_component_data_version(component.into());
        let changed =
            prev_data_version != data_version || Some(layout_version) != prev_layout_version;
        self.arch_data_versions.set(arch.id, data_version.unwrap());
        self.arch_layout_versions.set(arch.id, layout_version);
        changed
    }
}

pub struct ComponentToGpuSystem<T: ComponentValue + bytemuck::Pod> {
    format: GpuComponentFormat,
    source_archetypes: ArchetypeFilter,
    source_component: Component<T>,
    destination_component: GpuComponentId,
    changed: ArchChangeDetection,
}
impl<T: ComponentValue + bytemuck::Pod> ComponentToGpuSystem<T> {
    pub fn new(
        format: GpuComponentFormat,
        source_component: Component<T>,
        destination_component: GpuComponentId,
    ) -> Self {
        assert_eq!(format.size(), std::mem::size_of::<T>() as u64);
        Self {
            format,
            source_component,
            source_archetypes: ArchetypeFilter::new().incl(source_component),
            destination_component,
            changed: ArchChangeDetection::new(),
        }
    }
    pub fn with_arch_filter(mut self, arch_filter: ArchetypeFilter) -> Self {
        self.source_archetypes = arch_filter;
        self
    }
}
impl<T: ComponentValue + bytemuck::Pod> System<GpuWorldSyncEvent> for ComponentToGpuSystem<T> {
    fn run(&mut self, world: &mut World, _: &GpuWorldSyncEvent) {
        profiling::scope!("ComponentToGpuSystem.run");
        let gpu_world = world.resource(gpu_world()).lock();
        let gpu = world.resource(gpu()).clone();
        for arch in self.source_archetypes.iter_archetypes(world) {
            if let Some((gpu_buff, offset, layout_version)) =
                gpu_world.get_buffer(self.format, self.destination_component, arch.id)
            {
                if self
                    .changed
                    .changed(arch, self.source_component, layout_version)
                {
                    let buf = arch.get_component_buffer(self.source_component).unwrap();
                    gpu.queue
                        .write_buffer(gpu_buff, offset, bytemuck::cast_slice(&buf.data));
                }
            }
        }
    }
}
impl<T: ComponentValue + bytemuck::Pod> Debug for ComponentToGpuSystem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentToGpuSystem").finish()
    }
}

pub struct MappedComponentToGpuSystem<A: ComponentValue, B: bytemuck::Pod> {
    format: GpuComponentFormat,
    source_component: Component<A>,
    destination_component: GpuComponentId,
    map: Box<dyn Fn(&World, EntityId, &A) -> B + Sync + Send>,
    changed: ArchChangeDetection,
}
impl<A: ComponentValue, B: bytemuck::Pod> MappedComponentToGpuSystem<A, B> {
    pub fn new(
        format: GpuComponentFormat,
        source_component: Component<A>,
        destination_component: GpuComponentId,
        map: Box<dyn Fn(&World, EntityId, &A) -> B + Sync + Send>,
    ) -> Self {
        assert_eq!(format.size(), std::mem::size_of::<B>() as u64);
        Self {
            format,
            source_component,
            destination_component,
            map,
            changed: ArchChangeDetection::new(),
        }
    }
}
impl<A: ComponentValue, B: bytemuck::Pod> System<GpuWorldSyncEvent>
    for MappedComponentToGpuSystem<A, B>
{
    fn run(&mut self, world: &mut World, _: &GpuWorldSyncEvent) {
        profiling::scope!("MappedComponentToGpu.run");
        let gpu_world = world.resource(gpu_world()).lock();
        let gpu = world.resource(gpu());
        for arch in world.archetypes() {
            if let Some((gpu_buff, offset, layout_version)) =
                gpu_world.get_buffer(self.format, self.destination_component, arch.id)
            {
                if self
                    .changed
                    .changed(arch, self.source_component, layout_version)
                {
                    let buf = arch.get_component_buffer(self.source_component).unwrap();
                    let data = buf
                        .data
                        .iter()
                        .enumerate()
                        .map(&|(index, value)| {
                            (self.map)(world, arch.get_entity_id_from_index(index), value)
                        })
                        .collect_vec();
                    gpu.queue
                        .write_buffer(gpu_buff, offset, bytemuck::cast_slice(&data));
                }
            }
        }
    }
}
impl<A: ComponentValue, B: bytemuck::Pod> Debug for MappedComponentToGpuSystem<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MappedComponentToGpu").finish()
    }
}
