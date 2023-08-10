use ambient_core::{
    bounding::world_bounding_sphere,
    camera::{fovy, get_active_camera},
    hierarchy::children,
    main_scene,
    player::local_user_id,
    transform::translation,
};
use ambient_ecs::{components, query, ECSError, EntityId, Networked, Store, SystemGroup, World};
use ambient_gpu_ecs::{
    gpu_components, ComponentToGpuSystem, GpuComponentFormat, GpuWorldSyncEvent,
};
use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::primitives;
use ambient_gpu::gpu::Gpu;
use std::sync::Arc;

/// Maximum number of LOD levels
pub const MAX_LOD_LEVELS: usize = 16;
#[repr(transparent)]
/// Represents clip space size cutoffs for the lod levels.
///
/// Newtype enforces lod level count
#[derive(Default, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct LodCutoffs([f32; MAX_LOD_LEVELS]);

impl LodCutoffs {
    /// Construct lod levels, truncates and logs if `lods` exceed [`MAX_LOD_LEVELS`]
    pub fn new(lods: &[f32]) -> Self {
        let mut val = [0.0; MAX_LOD_LEVELS];
        if lods.len() > MAX_LOD_LEVELS {
            tracing::error!("Truncating lod levels {lods:?} to {MAX_LOD_LEVELS} items")
        }

        let len = lods.len().min(MAX_LOD_LEVELS);

        val[0..len].copy_from_slice(&lods[0..len]);

        Self(val)
    }
}

components!("rendering", {
    @[Networked, Store]
    lod_cutoffs: LodCutoffs,
    @[Networked, Store]
    cpu_lod: usize,
    @[Networked, Store]
    cpu_lod_group: (),
    @[Networked, Store]
    cpu_lod_visible: bool,
    /// Updated by the gpu
    /// Stores the computed current lod-level as calculated from the lod cutoffs
    @[Networked, Store]
    gpu_lod: (),
});
gpu_components! {
    lod_cutoffs(), gpu_lod() => lod_cutoffs: GpuComponentFormat::Mat4,
    // [lod, 0, 0, 0]
    gpu_lod() => gpu_lod: GpuComponentFormat::Vec4,
}

pub fn lod_system() -> SystemGroup {
    SystemGroup::new(
        "lod",
        vec![
            query((lod_cutoffs(), cpu_lod(), world_bounding_sphere())).to_system(
                |q, world, qs, _| {
                    if let Some(main_camera) =
                        get_active_camera(world, main_scene(), world.resource_opt(local_user_id()))
                    {
                        let camera_pos =
                            world.get(main_camera, translation()).unwrap_or(Vec3::ZERO);
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
                        for (id, (lod_cutoffs, &current_lod, bounding_sphere)) in q.iter(world, qs)
                        {
                            let dist = (camera_pos - bounding_sphere.center).length();
                            let clip_space_radius =
                                bounding_sphere.radius * main_camera_cot_fov_2 / dist;

                            let l = lod_cutoffs
                                .0
                                .iter()
                                .position(|x| clip_space_radius >= *x)
                                .unwrap_or(lod_cutoffs.0.len());
                            if l != current_lod {
                                to_update.push((id, l, current_lod));
                            }
                        }
                        for (id, l, current_lod) in to_update {
                            world.set(id, cpu_lod(), l).unwrap();
                            if world.has_component(id, cpu_lod_group()) {
                                if let Ok(children) =
                                    world.get_ref(id, children()).map(|x| x.clone())
                                {
                                    if current_lod < children.len() {
                                        set_lod_visible_recursive(
                                            world,
                                            children[current_lod],
                                            false,
                                        )
                                        .unwrap();
                                    }
                                    if l < children.len() {
                                        set_lod_visible_recursive(world, children[l], true)
                                            .unwrap();
                                    }
                                }
                            }
                        }
                    }
                },
            ),
        ],
    )
}

pub fn gpu_world_system(gpu: Arc<Gpu>) -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "lod/gpu_world",
        vec![Box::new(ComponentToGpuSystem::new(
            gpu,
            GpuComponentFormat::Mat4,
            lod_cutoffs(),
            gpu_components::lod_cutoffs(),
        ))],
    )
}

pub fn set_lod_visible_recursive(
    world: &mut World,
    id: EntityId,
    value: bool,
) -> Result<(), ECSError> {
    if world.has_component(id, primitives()) {
        world.set(id, cpu_lod_visible(), value)?;
    }
    let cs = world
        .get_ref(id, children())
        .map(|cs| cs.clone())
        .unwrap_or_default();
    for c in cs {
        set_lod_visible_recursive(world, c, value)?;
    }
    Ok(())
}
