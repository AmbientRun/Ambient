use std::{
    ops::Range,
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
use glam::{UVec4, Vec2, Vec4};
use itertools::Itertools;
use parking_lot::Mutex;
use wgpu::RenderPass;

use crate::{
    gpu::{Gpu, GpuKey},
    shader_module::ShaderModule,
    typed_buffer::TypedBuffer,
};

pub static MESH_BUFFER_TYPES_WGSL: &str = include_str!("mesh_buffer.wgsl");

pub fn get_mesh_buffer_types() -> ShaderModule {
    ShaderModule::from_str("MeshBufferTypes", include_str!("mesh_buffer.wgsl"))
}

static MESHES_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

pub type GpuMeshIndex = u64;

#[derive(Debug)]
pub struct GpuMesh {
    index: GpuMeshIndex,
    name: String,
    size_in_bytes: usize,
    // Notify parent to remove self on drop
    to_remove: Arc<Mutex<Vec<u64>>>,
}
impl GpuMesh {
    pub fn from_mesh(assets: AssetCache, mesh: &Mesh) -> Arc<GpuMesh> {
        MeshBufferKey.get(&assets).lock().insert(mesh)
    }
    pub fn name(&self) -> &str {
        &self.name
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
        Arc::new(Mutex::new(MeshBuffer::new(gpu)))
    }
}

#[derive(Clone, Debug)]
pub struct GpuMeshFromUrl {
    pub url: AbsAssetUrl,
    pub cache_on_disk: bool,
}
impl GpuMeshFromUrl {
    pub fn new(url: impl AsRef<str>, cache_on_disk: bool) -> anyhow::Result<Self> {
        Ok(Self { url: AbsAssetUrl::parse(url)?, cache_on_disk })
    }
}
#[async_trait]
impl AsyncAssetKey<AssetResult<Arc<GpuMesh>>> for GpuMeshFromUrl {
    async fn load(self, assets: AssetCache) -> AssetResult<Arc<GpuMesh>> {
        let mesh = MeshFromUrl::new(self.url, self.cache_on_disk).get(&assets).await?;
        Ok(GpuMesh::from_mesh(assets, &mesh))
    }
}

/// Gpu mesh buffer which holds all meshes in an Elements application.
///
/// A GpuMesh in the application just keeps an index into the metadata_buffer, and
/// from there the shaders can look up exactly where to read position, normal etc.
/// The "id"s (GpuMesh.index) are recycled, so even when a mesh is dropped and removed
/// from the application, all current GpuMesh.index's are still valid (and the content
/// of the metadata is just updated at the index).
pub struct MeshBuffer {
    gpu: Arc<Gpu>,
    pub metadata_buffer: TypedBuffer<MeshMetadata>,
    pub position_buffer: AttributeBuffer<Vec4>, // Vec4 instead of Vec3 because of alignment (16)
    pub normal_buffer: AttributeBuffer<Vec4>,
    pub tangent_buffer: AttributeBuffer<Vec4>,
    pub texcoord0_buffer: AttributeBuffer<Vec2>,
    pub joint_buffer: AttributeBuffer<UVec4>,
    pub weight_buffer: AttributeBuffer<Vec4>,
    pub index_buffer: AttributeBuffer<u32>,
    meshes: Vec<Option<InternalMesh>>,
    to_remove: Arc<Mutex<Vec<GpuMeshIndex>>>,
    free_indices: Vec<GpuMeshIndex>,
}
impl MeshBuffer {
    pub fn new(gpu: Arc<Gpu>) -> Self {
        Self {
            metadata_buffer: TypedBuffer::new(
                gpu.clone(),
                "MeshBuffer.metadata_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            index_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.index_buffer",
                1,
                0,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            position_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.position_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            normal_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.normal_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            tangent_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.tangent_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            texcoord0_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.texcoord0_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            joint_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.joint_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            weight_buffer: AttributeBuffer::new(
                gpu.clone(),
                "MeshBuffer.weight_buffer",
                1,
                0,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            ),
            meshes: Vec::new(),
            to_remove: Arc::new(Mutex::new(Vec::new())),
            free_indices: Vec::new(),
            gpu,
        }
    }
    pub fn insert(&mut self, mesh: &Mesh) -> Arc<GpuMesh> {
        let metadata = MeshMetadata {
            position_offset: self.position_buffer.front.len() as u32,
            normal_offset: self.normal_buffer.front.len() as u32,
            tangent_offset: self.tangent_buffer.front.len() as u32,
            texcoord0_offset: self.texcoord0_buffer.front.len() as u32,
            joint_offset: self.joint_buffer.front.len() as u32,
            weight_offset: self.weight_buffer.front.len() as u32,
            index_offset: self.index_buffer.front.len() as u32,
            index_count: mesh.indices.as_ref().map(|x| x.len()).unwrap_or_default() as u32,
        };

        let mut internal_mesh = InternalMesh { metadata, ..Default::default() };
        if let Some(positions) = &mesh.positions {
            self.position_buffer.front.resize(self.position_buffer.front.len() + positions.len() as u64, true);
            self.position_buffer.front.write(metadata.position_offset as u64, &positions.iter().map(|p| p.extend(0.)).collect_vec());
            internal_mesh.position_count = positions.len() as u64;
        }
        if let Some(normals) = &mesh.normals {
            self.normal_buffer.front.resize(self.normal_buffer.front.len() + normals.len() as u64, true);
            self.normal_buffer.front.write(metadata.normal_offset as u64, &normals.iter().map(|p| p.extend(0.)).collect_vec());
            internal_mesh.normal_count = normals.len() as u64;
        }
        if let Some(tangents) = &mesh.tangents {
            self.tangent_buffer.front.resize(self.tangent_buffer.front.len() + tangents.len() as u64, true);
            self.tangent_buffer.front.write(metadata.tangent_offset as u64, &tangents.iter().map(|p| p.extend(0.)).collect_vec());
            internal_mesh.tangent_count = tangents.len() as u64;
        }
        if let Some(texcoord0s) = &mesh.texcoords.get(0) {
            self.texcoord0_buffer.front.resize(self.texcoord0_buffer.front.len() + texcoord0s.len() as u64, true);
            self.texcoord0_buffer.front.write(metadata.texcoord0_offset as u64, texcoord0s);
            internal_mesh.texcoord0_count = texcoord0s.len() as u64;
        }
        if let Some(joints) = &mesh.joint_indices {
            self.joint_buffer.front.resize(self.joint_buffer.front.len() + joints.len() as u64, true);
            self.joint_buffer.front.write(metadata.joint_offset as u64, joints);
            internal_mesh.joint_count = joints.len() as u64;
        }
        if let Some(weights) = &mesh.joint_weights {
            self.weight_buffer.front.resize(self.weight_buffer.front.len() + weights.len() as u64, true);
            self.weight_buffer.front.write(metadata.weight_offset as u64, weights);
            internal_mesh.weight_count = weights.len() as u64;
        }
        if let Some(indices) = &mesh.indices {
            self.index_buffer.front.resize(self.index_buffer.front.len() + indices.len() as u64, true);
            self.index_buffer.front.write(metadata.index_offset as u64, indices);
            internal_mesh.index_count = indices.len() as u64;
        }

        let metadata_offset = if let Some(offset) = self.free_indices.pop() {
            self.meshes[offset as usize] = Some(internal_mesh);
            offset
        } else {
            let offset = self.metadata_buffer.len();
            self.metadata_buffer.resize(self.metadata_buffer.len() + 1, true);
            self.meshes.push(Some(internal_mesh));
            offset
        };
        self.metadata_buffer.write(metadata_offset, &[metadata]);
        MESHES_TOTAL_SIZE.store(self.size() as usize, Ordering::SeqCst);
        Arc::new(GpuMesh {
            index: metadata_offset,
            name: mesh.name.clone(),
            size_in_bytes: mesh.size_in_bytes(),
            to_remove: self.to_remove.clone(),
        })
    }
    pub fn update(&mut self) {
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
            .sorted_by_key(|index| self.meshes[**index as usize].as_ref().unwrap().metadata.position_offset)
            .next()
            .unwrap();

        let base_offset = self.meshes[first_to_remove_mesh_index as usize].as_ref().unwrap().metadata;

        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("MeshBuffer") });
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
            .filter(|(_, x)| x.metadata.position_offset >= base_offset.position_offset)
            .collect_vec();
        update_meshes_sorted.sort_by_key(|(_, x)| x.metadata.position_offset);

        let mut sizes = MeshMetadata::default();
        for (_, mesh) in &update_meshes_sorted {
            sizes.position_offset += mesh.position_count as u32;
            sizes.normal_offset += mesh.normal_count as u32;
            sizes.tangent_offset += mesh.tangent_count as u32;
            sizes.texcoord0_offset += mesh.texcoord0_count as u32;
            sizes.joint_offset += mesh.joint_count as u32;
            sizes.weight_offset += mesh.weight_count as u32;
            sizes.index_offset += mesh.index_count as u32;
        }
        self.position_buffer.tmp.resize(sizes.position_offset as u64, true);
        self.normal_buffer.tmp.resize(sizes.normal_offset as u64, true);
        self.tangent_buffer.tmp.resize(sizes.tangent_offset as u64, true);
        self.texcoord0_buffer.tmp.resize(sizes.texcoord0_offset as u64, true);
        self.joint_buffer.tmp.resize(sizes.joint_offset as u64, true);
        self.weight_buffer.tmp.resize(sizes.weight_offset as u64, true);
        self.index_buffer.tmp.resize(sizes.index_offset as u64, true);

        let mut cursor = MeshMetadata::default();
        for (index, mesh) in update_meshes_sorted {
            self.meshes[index].as_mut().unwrap().metadata = MeshMetadata {
                index_count: mesh.index_count as u32,
                position_offset: base_offset.position_offset + cursor.position_offset,
                normal_offset: base_offset.normal_offset + cursor.normal_offset,
                tangent_offset: base_offset.tangent_offset + cursor.tangent_offset,
                texcoord0_offset: base_offset.texcoord0_offset + cursor.texcoord0_offset,
                joint_offset: base_offset.joint_offset + cursor.joint_offset,
                weight_offset: base_offset.weight_offset + cursor.weight_offset,
                index_offset: base_offset.index_offset + cursor.index_offset,
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
            copy_buff!(encoder, mesh, cursor, position_buffer, position_offset, position_count);
            copy_buff!(encoder, mesh, cursor, normal_buffer, normal_offset, normal_count);
            copy_buff!(encoder, mesh, cursor, tangent_buffer, tangent_offset, tangent_count);
            copy_buff!(encoder, mesh, cursor, texcoord0_buffer, texcoord0_offset, texcoord0_count);
            copy_buff!(encoder, mesh, cursor, joint_buffer, joint_offset, joint_count);
            copy_buff!(encoder, mesh, cursor, weight_buffer, weight_offset, weight_count);
            copy_buff!(encoder, mesh, cursor, index_buffer, index_offset, index_count);
        }

        macro_rules! copy_back_buff {
            ( $encoder:expr, $base_offset:ident, $buff:ident, $field:ident ) => {
                self.$buff.front.resize($base_offset.$field as u64 + self.$buff.tmp.len(), true);
                encoder.copy_buffer_to_buffer(
                    self.$buff.tmp.buffer(),
                    0,
                    self.$buff.front.buffer(),
                    $base_offset.$field as u64 * self.$buff.front.item_size(),
                    self.$buff.tmp.size(),
                );
            };
        }
        copy_back_buff!(encoder, base_offset, position_buffer, position_offset);
        copy_back_buff!(encoder, base_offset, normal_buffer, normal_offset);
        copy_back_buff!(encoder, base_offset, tangent_buffer, tangent_offset);
        copy_back_buff!(encoder, base_offset, texcoord0_buffer, texcoord0_offset);
        copy_back_buff!(encoder, base_offset, joint_buffer, joint_offset);
        copy_back_buff!(encoder, base_offset, weight_buffer, weight_offset);
        copy_back_buff!(encoder, base_offset, index_buffer, index_offset);
        let metadata = self.meshes.iter().map(|mesh| mesh.as_ref().map(|x| x.metadata).unwrap_or_default()).collect_vec();
        self.metadata_buffer.write(0, &metadata);

        self.gpu.queue.submit(Some(encoder.finish()));
        MESHES_TOTAL_SIZE.store(self.size() as usize, Ordering::SeqCst);
    }
    pub fn get_mesh_metadata(&self, mesh: &GpuMesh) -> &MeshMetadata {
        &self.meshes[mesh.index as usize].as_ref().unwrap().metadata
    }
    pub fn size(&self) -> u64 {
        self.metadata_buffer.size()
            + self.position_buffer.front.size()
            + self.normal_buffer.front.size()
            + self.tangent_buffer.front.size()
            + self.texcoord0_buffer.front.size()
            + self.joint_buffer.front.size()
            + self.weight_buffer.front.size()
            + self.index_buffer.front.size()
    }
    pub fn n_meshes(&self) -> usize {
        self.meshes.len() - self.free_indices.len()
    }
    pub fn total_bytes_used() -> usize {
        MESHES_TOTAL_SIZE.load(Ordering::SeqCst)
    }

    pub fn bind<'a>(&'a self, renderpass: &'a mut RenderPass<'a>) {
        renderpass.set_index_buffer(self.index_buffer.buffer().slice(..), wgpu::IndexFormat::Uint32)
    }

    pub fn indices_of(&self, mesh: &GpuMesh) -> Range<u32> {
        let mesh = self.get_mesh_metadata(mesh);
        mesh.index_offset..(mesh.index_offset + mesh.index_count)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshMetadata {
    pub position_offset: u32,
    pub normal_offset: u32,
    pub tangent_offset: u32,
    pub texcoord0_offset: u32,
    pub joint_offset: u32,
    pub weight_offset: u32,
    pub index_offset: u32,

    pub index_count: u32,
}

#[derive(Debug, Clone, Default)]
struct InternalMesh {
    metadata: MeshMetadata,
    position_count: u64,
    normal_count: u64,
    tangent_count: u64,
    texcoord0_count: u64,
    joint_count: u64,
    weight_count: u64,
    index_count: u64,
}

pub struct AttributeBuffer<T: bytemuck::Pod> {
    pub front: TypedBuffer<T>,
    pub tmp: TypedBuffer<T>,
}
impl<T: bytemuck::Pod> AttributeBuffer<T> {
    pub fn new(gpu: Arc<Gpu>, label: &str, capacity: u64, length: u64, usage: wgpu::BufferUsages) -> Self {
        Self {
            front: TypedBuffer::new(gpu.clone(), label, capacity, length, usage),
            tmp: TypedBuffer::new(gpu, label, capacity, length, usage),
        }
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.front.buffer()
    }
}
