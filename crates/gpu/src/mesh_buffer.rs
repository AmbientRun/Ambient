use std::{
    ops::Range,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use ambient_std::{
    asset_cache::{AssetCache, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKey, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    download_asset::{AssetResult, MeshFromUrl},
    mesh::Mesh,
};
use async_trait::async_trait;
use bytemuck::{Pod, Zeroable};
use glam::{UVec4, Vec2, Vec4};
use itertools::Itertools;
use parking_lot::Mutex;
use wgpu::RenderPass;

use crate::{
    gpu::{Gpu, GpuKey},
    typed_buffer::TypedBuffer,
};

static MESHES_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

pub type GpuMeshIndex = u32;

pub struct GpuMesh {
    index: GpuMeshIndex,
    size_in_bytes: usize,
    // Notify parent to remove self on drop
    to_remove: Arc<Mutex<Vec<GpuMeshIndex>>>,
}

impl std::fmt::Debug for GpuMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("GpuMesh");

        s.field("index", &self.index)
            .field("size_in_bytes", &self.size_in_bytes)
            .finish_non_exhaustive()
    }
}

impl GpuMesh {
    pub fn from_mesh(gpu: &Gpu, assets: &AssetCache, mesh: &Mesh) -> Arc<GpuMesh> {
        MeshBufferKey.get(assets).lock().insert(gpu, mesh)
    }
    pub fn index(&self) -> GpuMeshIndex {
        self.index
    }
    pub fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }
}

impl Drop for GpuMesh {
    fn drop(&mut self) {
        self.to_remove.lock().push(self.index);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshBufferKey;

impl SyncAssetKey<Arc<Mutex<MeshBuffer>>> for MeshBufferKey {
    fn load(&self, assets: AssetCache) -> Arc<Mutex<MeshBuffer>> {
        let gpu = GpuKey.get(&assets);
        Arc::new(Mutex::new(MeshBuffer::new(&gpu)))
    }
}

#[derive(Clone, Debug)]
pub struct GpuMeshFromUrl {
    pub url: AbsAssetUrl,
    pub cache_on_disk: bool,
}

impl GpuMeshFromUrl {
    pub fn new(url: impl AsRef<str>, cache_on_disk: bool) -> anyhow::Result<Self> {
        Ok(Self {
            url: AbsAssetUrl::from_str(url.as_ref())?,
            cache_on_disk,
        })
    }
}

#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<GpuMesh>>> for GpuMeshFromUrl {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<GpuMesh>> {
        let gpu = GpuKey.get(&assets);
        let mesh = MeshFromUrl::new(self.url, self.cache_on_disk)
            .get(&assets)
            .await?;
        Ok(GpuMesh::from_mesh(&gpu, &assets, &mesh))
    }
}

/// Groups all *common* mesh attributes into a single struct to reduce the number of bound slots.
#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct BaseMesh {
    position: Vec4,
    normal: Vec4,
    tangent: Vec4,
    texcoord0: Vec2,
    _padding: Vec2,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Pod, Zeroable)]
pub struct SkinnedMesh {
    joint: UVec4,
    weights: Vec4,
}

/// Gpu mesh buffer which holds all meshes in an Elements application.
///
/// A GpuMesh in the application just keeps an index into the metadata_buffer, and
/// from there the shaders can look up exactly where to read position, normal etc.
/// The "id"s (GpuMesh.index) are recycled, so even when a mesh is dropped and removed
/// from the application, all current GpuMesh.index's are still valid (and the content
/// of the metadata is just updated at the index).
pub struct MeshBuffer {
    pub metadata_buffer: TypedBuffer<MeshMetadata>,
    pub base_buffer: AttributeBuffer<BaseMesh>,
    pub skinned_buffer: AttributeBuffer<SkinnedMesh>,

    pub index_buffer: AttributeBuffer<u32>,
    meshes: Vec<Option<InternalMesh>>,
    to_remove: Arc<Mutex<Vec<GpuMeshIndex>>>,
    free_indices: Vec<GpuMeshIndex>,
}

impl MeshBuffer {
    pub fn new(gpu: &Gpu) -> Self {
        Self {
            metadata_buffer: TypedBuffer::new(
                gpu,
                "MeshBuffer.metadata_buffer",
                4,
                0,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            ),
            index_buffer: AttributeBuffer::new(
                gpu,
                "MeshBuffer.index_buffer",
                4,
                0,
                wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            ),
            base_buffer: AttributeBuffer::new(
                gpu,
                "MeshBuffer.base_buffer",
                4,
                0,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            ),

            skinned_buffer: AttributeBuffer::new(
                gpu,
                "MeshBuffer.skinned_buffer",
                4,
                0,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            ),
            meshes: Vec::new(),
            to_remove: Arc::new(Mutex::new(Vec::new())),
            free_indices: Vec::new(),
        }
    }

    pub fn insert(&mut self, gpu: &Gpu, mesh: &Mesh) -> Arc<GpuMesh> {
        let metadata = MeshMetadata {
            base_offset: self.base_buffer.front.len() as u32,
            skinned_offset: self.skinned_buffer.front.len() as u32,
            index_offset: self.index_buffer.front.len() as u32,
            index_count: mesh.index_count(),
        };

        let mut internal_mesh = InternalMesh {
            metadata,
            ..Default::default()
        };

        // Pad all vertex attributes to match vertex positions buffer.
        {
            let pos = mesh.positions();
            let norm = mesh.normals();
            let tan = mesh.tangents();
            let uv = mesh.texcoords(0);

            let len = ([pos.len(), norm.len(), tan.len(), uv.len()])
                .into_iter()
                .max()
                .unwrap_or(0);

            let mut data = vec![BaseMesh::default(); len];

            pos.iter()
                .zip(&mut data)
                .for_each(|(src, dst)| dst.position = src.extend(0.0));
            norm.iter()
                .zip(&mut data)
                .for_each(|(src, dst)| dst.normal = src.extend(0.0));
            tan.iter()
                .zip(&mut data)
                .for_each(|(src, dst)| dst.tangent = src.extend(0.0));
            uv.iter()
                .zip(&mut data)
                .for_each(|(src, dst)| dst.texcoord0 = *src);

            self.base_buffer
                .front
                .resize(gpu, self.base_buffer.front.len() + data.len(), true);

            self.base_buffer
                .front
                .write(gpu, metadata.base_offset as usize, &data);

            internal_mesh.base_count += data.len() as u64;
        }

        if !mesh.joint_indices().is_empty() && !mesh.joint_weights().is_empty() {
            let joints = mesh.joint_indices();
            let weights = mesh.joint_weights();

            let len = joints.len().max(weights.len());

            let mut data = vec![SkinnedMesh::default(); len];

            joints
                .iter()
                .zip(&mut data)
                .for_each(|(src, dst)| dst.joint = *src);
            weights
                .iter()
                .zip(&mut data)
                .for_each(|(src, dst)| dst.weights = *src);

            self.skinned_buffer
                .front
                .resize(gpu, self.skinned_buffer.front.len() + len, true);

            self.skinned_buffer
                .front
                .write(gpu, metadata.skinned_offset as usize, &data);

            internal_mesh.skinned_count += len as u64;
        }

        self.index_buffer.front.resize(
            gpu,
            self.index_buffer.front.len() + mesh.index_count() as usize,
            true,
        );

        self.index_buffer
            .front
            .write(gpu, metadata.index_offset as usize, mesh.indices());

        internal_mesh.index_count = mesh.index_count().try_into().unwrap();

        let metadata_offset = if let Some(offset) = self.free_indices.pop() {
            self.meshes[offset as usize] = Some(internal_mesh);
            offset as usize
        } else {
            let offset = self.metadata_buffer.len();
            self.metadata_buffer
                .resize(gpu, self.metadata_buffer.len() + 1, true);
            self.meshes.push(Some(internal_mesh));
            offset
        };

        self.metadata_buffer
            .write(gpu, metadata_offset, &[metadata]);
        MESHES_TOTAL_SIZE.store(self.size() as usize, Ordering::SeqCst);

        Arc::new(GpuMesh {
            index: metadata_offset as u32,
            size_in_bytes: mesh.size_in_bytes(),
            to_remove: self.to_remove.clone(),
        })
    }

    pub fn update(&mut self, gpu: &Gpu) {
        let to_remove = {
            let mut to_remove = self.to_remove.lock();
            to_remove.drain(..).collect_vec()
        };

        if to_remove.is_empty() {
            return;
        }

        // We let the meshes before the first removed mesh just remain; no need to copy them around
        let first_to_remove_mesh_index = *to_remove
            .iter()
            .sorted_by_key(|index| {
                self.meshes[**index as usize]
                    .as_ref()
                    .unwrap()
                    .metadata
                    .base_offset
            })
            .next()
            .unwrap();

        let base_metadata = self.meshes[first_to_remove_mesh_index as usize]
            .as_ref()
            .unwrap()
            .metadata;

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("MeshBuffer"),
            });
        for index in to_remove {
            self.meshes[index as usize] = None;
            self.free_indices.push(index);
        }
        let mut update_meshes_sorted = self
            .meshes
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(i, x)| x.map(|x| (i, x)))
            .filter(|(_, x)| x.metadata.base_offset >= base_metadata.base_offset)
            .collect_vec();

        update_meshes_sorted.sort_by_key(|(_, x)| x.metadata.base_offset);

        let mut sizes = MeshMetadata::default();
        for (_, mesh) in &update_meshes_sorted {
            sizes.base_offset += mesh.base_count as u32;
            sizes.skinned_offset += mesh.skinned_count as u32;
            sizes.index_offset += mesh.index_count as u32;
        }

        self.base_buffer
            .tmp
            .resize(gpu, sizes.base_offset as usize, true);

        self.skinned_buffer
            .tmp
            .resize(gpu, sizes.skinned_offset as usize, true);

        self.index_buffer
            .tmp
            .resize(gpu, sizes.index_offset as usize, true);

        let mut cursor = MeshMetadata::default();
        for (index, mesh) in update_meshes_sorted {
            self.meshes[index].as_mut().unwrap().metadata = MeshMetadata {
                index_count: mesh.index_count as u32,
                base_offset: base_metadata.base_offset + cursor.base_offset,
                skinned_offset: base_metadata.skinned_offset + cursor.skinned_offset,
                index_offset: base_metadata.index_offset + cursor.index_offset,
            };

            macro_rules! copy_buff {
                ( $encoder:expr, $mesh:expr, $cursor:expr, $buff:ident, $offset_field:ident, $count_field:ident ) => {
                    if $mesh.$count_field > 0 {
                        encoder.copy_buffer_to_buffer(
                            self.$buff.front.buffer(),
                            $mesh.metadata.$offset_field as u64 * self.$buff.front.item_size(),
                            self.$buff.tmp.buffer(),
                            $cursor.$offset_field as u64 * self.$buff.front.item_size(),
                            $mesh.$count_field * self.$buff.front.item_size(),
                        );
                        $cursor.$offset_field += $mesh.$count_field as u32;
                    }
                };
            }

            copy_buff!(encoder, mesh, cursor, base_buffer, base_offset, base_count);
            copy_buff!(
                encoder,
                mesh,
                cursor,
                skinned_buffer,
                skinned_offset,
                skinned_count
            );
            copy_buff!(
                encoder,
                mesh,
                cursor,
                index_buffer,
                index_offset,
                index_count
            );
        }

        macro_rules! copy_back_buff {
            ( $gpu:expr, $encoder:expr, $base_offset:ident, $buff:ident, $field:ident ) => {
                self.$buff.front.resize(
                    $gpu,
                    $base_offset.$field as usize + self.$buff.tmp.len(),
                    true,
                );
                let offset = $base_offset.$field as u64 * self.$buff.front.item_size();
                encoder.copy_buffer_to_buffer(
                    self.$buff.tmp.buffer(),
                    0,
                    self.$buff.front.buffer(),
                    offset,
                    self.$buff.tmp.byte_size() - offset,
                );
            };
        }

        copy_back_buff!(gpu, encoder, base_metadata, base_buffer, base_offset);
        copy_back_buff!(gpu, encoder, base_metadata, skinned_buffer, skinned_offset);
        copy_back_buff!(gpu, encoder, base_metadata, index_buffer, index_offset);

        let metadata = self
            .meshes
            .iter()
            .map(|mesh| mesh.as_ref().map(|x| x.metadata).unwrap_or_default())
            .collect_vec();
        self.metadata_buffer.write(gpu, 0, &metadata);

        gpu.queue.submit(Some(encoder.finish()));
        MESHES_TOTAL_SIZE.store(self.size() as usize, Ordering::SeqCst);
    }

    pub fn get_mesh_metadata(&self, mesh: &GpuMesh) -> &MeshMetadata {
        &self.meshes[mesh.index as usize].as_ref().unwrap().metadata
    }

    pub fn size(&self) -> u64 {
        self.metadata_buffer.byte_size()
            + self.base_buffer.front.byte_size()
            + self.skinned_buffer.front.byte_size()
            + self.index_buffer.front.byte_size()
    }

    pub fn n_meshes(&self) -> usize {
        self.meshes.len() - self.free_indices.len()
    }

    pub fn total_bytes_used() -> usize {
        MESHES_TOTAL_SIZE.load(Ordering::SeqCst)
    }

    pub fn bind<'a>(&'a self, renderpass: &'a mut RenderPass<'a>) {
        renderpass.set_index_buffer(
            self.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        )
    }

    pub fn indices_of(&self, mesh: &GpuMesh) -> Range<u32> {
        let mesh = self.get_mesh_metadata(mesh);
        mesh.index_offset..(mesh.index_offset + mesh.index_count)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshMetadata {
    /// position, normal, tangent, texcoord0 are grouped
    pub base_offset: u32,
    pub skinned_offset: u32,
    pub index_offset: u32,

    pub index_count: u32,
}

#[derive(Debug, Clone, Default)]
struct InternalMesh {
    metadata: MeshMetadata,
    base_count: u64,
    skinned_count: u64,
    index_count: u64,
}

pub struct AttributeBuffer<T: bytemuck::Pod> {
    pub front: TypedBuffer<T>,
    pub tmp: TypedBuffer<T>,
}

impl<T: bytemuck::Pod> AttributeBuffer<T> {
    pub fn new(
        gpu: &Gpu,
        label: &str,
        capacity: usize,
        length: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        Self {
            front: TypedBuffer::new(gpu, label, capacity, length, usage),
            tmp: TypedBuffer::new(gpu, label, capacity, length, usage),
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.front.buffer()
    }
}
