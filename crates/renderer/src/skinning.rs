use std::sync::{
    atomic::{AtomicU32, Ordering}, Arc
};

use elements_core::{
    asset_cache, gpu_components, gpu_ecs::{GpuComponentFormat, GpuWorldSyncEvent, MappedComponentToGpuSystem}, transform::{inv_local_to_world, local_to_world}
};
use elements_ecs::{components, query, Commands, EntityId, SystemGroup};
use elements_gpu::{
    gpu::{Gpu, GpuKey}, typed_buffer::TypedBuffer
};
use elements_std::asset_cache::{AssetCache, SyncAssetKey, SyncAssetKeyExt};
use glam::Mat4;
use itertools::Itertools;
use parking_lot::Mutex;

components!("rendering", {
    inverse_bind_matrices: Arc<Vec<glam::Mat4>>,
    joints: Vec<EntityId>,
    joint_matrices: Vec<glam::Mat4>,

    skin: Skin,

    joints_by_fbx_id: Vec<i64>,
});
gpu_components! {
    skin() => skin: GpuComponentFormat::U32,
}

#[derive(Debug, Clone)]
pub struct Skin(Arc<AtomicU32>);
impl Skin {
    pub fn get_offset(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }
    pub fn null() -> Self {
        Self(Arc::new(AtomicU32::new(0)))
    }
}

#[derive(Debug, Clone)]
pub struct SkinsBufferKey;
impl SyncAssetKey<Arc<Mutex<SkinsBuffer>>> for SkinsBufferKey {
    fn load(&self, assets: AssetCache) -> Arc<Mutex<SkinsBuffer>> {
        let gpu = GpuKey.get(&assets);
        Arc::new(Mutex::new(SkinsBuffer::new(gpu)))
    }
}

// TODO: The skins are currently leaking memory as they are never cleaned up. Need to implement something similar to how MeshBuffer
// works; keep an index buffer and a data buffer, and re-use indices
pub struct SkinsBuffer {
    pub buffer: TypedBuffer<Mat4>,
}
impl SkinsBuffer {
    fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            buffer: TypedBuffer::new(
                gpu,
                "SkinsBuffer.buffer",
                1,
                1,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            ),
        }
    }
    pub fn create(&mut self, size: u32) -> Skin {
        let skin = Skin(Arc::new(AtomicU32::new(self.buffer.len() as u32)));
        self.buffer.resize(self.buffer.len() + size as u64, true);
        skin
    }
    pub fn update(&self, skin: &Skin, joint_matrices: &[Mat4]) {
        self.buffer.write(skin.get_offset() as u64, joint_matrices);
    }
}

pub fn skinning_systems() -> SystemGroup {
    SystemGroup::new(
        "skinning_systems",
        vec![query((inv_local_to_world(), inverse_bind_matrices(), joints(), skin())).to_system(|q, world, qs, _| {
            let skins_h = SkinsBufferKey.get(world.resource(asset_cache()));
            let skins = skins_h.lock();
            let mut commands = Commands::new();
            for (id, (&inv_local_to_world, inverse_bind_matrices, joints, skin)) in q.iter(world, qs) {
                let joint_matrices = joints
                    .iter()
                    .enumerate()
                    .map(|(i, joint)| {
                        inv_local_to_world
                            * world.get(*joint, local_to_world()).unwrap()
                            * *inverse_bind_matrices.get(i).unwrap_or(&glam::Mat4::IDENTITY)
                    })
                    .collect_vec();
                skins.update(skin, &joint_matrices);
                commands.set(id, self::joint_matrices(), joint_matrices);
            }
            commands.apply(world).unwrap();
        })],
    )
}

pub fn gpu_world_systems() -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "skinning/gpu_world",
        vec![Box::new(MappedComponentToGpuSystem::new(
            GpuComponentFormat::U32,
            skin(),
            gpu_components::skin(),
            Box::new(|_, _, skin| skin.get_offset()),
        ))],
    )
}
