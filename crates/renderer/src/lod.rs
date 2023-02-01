use std::sync::Arc;

use elements_core::{
    bounding::world_bounding_sphere, camera::{fovy, get_active_camera}, gpu_components, gpu_ecs::{ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent}, hierarchy::children, main_scene, transform::translation
};
use elements_ecs::{components, query, ECSError, EntityId, Networked, Store, SystemGroup, World};
use elements_gpu::mesh_buffer::GpuMesh;
use glam::Vec3;

use crate::primitives;

components!("rendering", {
    mesh_lods: Vec<Arc<GpuMesh>>,
    @[Networked, Store]
    lod_cutoffs: [f32; 20],
    @[Networked, Store]
    cpu_lod: usize,
    @[Networked, Store]
    cpu_lod_group: (),
    @[Networked, Store]
    cpu_lod_visible: bool,
    @[Networked, Store]
    gpu_lod: (),
});
gpu_components! {
    lod_cutoffs(), gpu_lod() => lod_cutoffs: GpuComponentFormat::F32Array20,
    gpu_lod() => gpu_lod: GpuComponentFormat::U32,
}

pub fn lod_system() -> SystemGroup {
    SystemGroup::new(
        "lod",
        vec![
            query((lod_cutoffs(), cpu_lod(), world_bounding_sphere())).to_system(|q, world, qs, _| {
                if let Some(main_camera) = get_active_camera(world, main_scene()) {
                    let camera_pos = world.get(main_camera, translation()).unwrap_or(Vec3::ZERO);
                    let main_camera_fov = match world.get(main_camera, fovy()) {
                        Ok(val) => val,
                        Err(_) => return,
                    };
                    let main_camera_cot_fov_2 = 1. / (main_camera_fov / 2.).tan();

                    // let frame = world.resource(frame_index());
                    // let count = q.query.iter(world, None).count();
                    // let chunk_size = (count / 100).max(1);
                    // let chunks = ((count as f32 / chunk_size as f32).ceil() as usize).max(1);
                    // let start = (frame % chunks) * chunk_size;

                    let mut to_update = Vec::new();
                    for (id, (lod_cutoffs, &current_lod, bounding_sphere)) in q.iter(world, qs) {
                        let dist = (camera_pos - bounding_sphere.center).length();
                        let clip_space_radius = bounding_sphere.radius * main_camera_cot_fov_2 / dist;

                        let l = lod_cutoffs.iter().position(|x| clip_space_radius >= *x).unwrap_or(lod_cutoffs.len());
                        if l != current_lod {
                            to_update.push((id, l, current_lod));
                        }
                    }
                    for (id, l, current_lod) in to_update {
                        world.set(id, cpu_lod(), l).unwrap();
                        if world.has_component(id, cpu_lod_group()) {
                            if let Ok(children) = world.get_ref(id, children()).map(|x| x.clone()) {
                                if current_lod < children.len() {
                                    set_lod_visible_recursive(world, children[current_lod], false).unwrap();
                                }
                                if l < children.len() {
                                    set_lod_visible_recursive(world, children[l], true).unwrap();
                                }
                            }
                        }
                    }
                }
            }),
            // query_mut((mesh(),), (mesh_lods(), cpu_lod().changed())).to_system(|q, world, qs, _| {
            //     for (_, (mesh,), (mesh_lods, &lod)) in q.iter(world, qs) {
            //         *mesh = mesh_lods[lod].clone();
            //     }
            // }),
        ],
    )
}

pub fn gpu_world_system() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "lod/gpu_world",
        vec![Box::new(ComponentToGpuSystem::new(GpuComponentFormat::F32Array20, lod_cutoffs(), gpu_components::lod_cutoffs()))],
    )
}

pub fn set_lod_visible_recursive(world: &mut World, id: EntityId, value: bool) -> Result<(), ECSError> {
    if world.has_component(id, primitives()) {
        world.set(id, cpu_lod_visible(), value)?;
    }
    let cs = world.get_ref(id, children()).map(|cs| cs.clone()).unwrap_or_default();
    for c in cs {
        set_lod_visible_recursive(world, c, value)?;
    }
    Ok(())
}
