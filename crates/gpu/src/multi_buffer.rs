use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use thiserror::Error;

use super::gpu::Gpu;

#[derive(Clone, Debug, Error)]
pub enum MultiBufferError {
    #[error("No such sub buffer: {0}")]
    NoSuchSubBuffer(SubBufferId),
    #[error("Write out of range, trying to write offset {offset} and length {data_len} into buffer of length {buffer_len}")]
    WriteOfRange { offset: u64, data_len: u64, buffer_len: u64 },
}

pub enum MultiBufferSizeStrategy {
    /// The sub-buffers are exactly the requested size
    Exact,
    /// The sub-buffers are padded to a power of 2 of the requested size
    Pow2,
}

static MULTI_BUFFERS_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

pub struct MultiBuffer {
    gpu: Arc<Gpu>,
    buffer: wgpu::Buffer,
    sub_buffers: Vec<Option<SubBuffer>>,
    free_ids: Vec<SubBufferId>,
    label: String,
    usage: wgpu::BufferUsages,
    total_capacity: u64,
    size_strategy: MultiBufferSizeStrategy,
}
pub type SubBufferId = usize;

impl MultiBuffer {
    pub fn new(gpu: Arc<Gpu>, label: &str, usage: wgpu::BufferUsages, size_strategy: MultiBufferSizeStrategy) -> Self {
        Self {
            buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), usage, size: 4, mapped_at_creation: false }),
            sub_buffers: Vec::new(),
            free_ids: Vec::new(),
            label: label.into(),
            usage,
            total_capacity: 0,
            size_strategy,
            gpu,
        }
    }
    pub fn total_bytes_used() -> usize {
        MULTI_BUFFERS_TOTAL_SIZE.load(Ordering::SeqCst)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn total_capacity_in_bytes(&self) -> u64 {
        self.total_capacity
    }

    pub fn create_buffer(&mut self, capacity: Option<u64>) -> SubBufferId {
        let mut encoder =
            self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("MultiBuffer.create_buffer") });
        let res = self.create_buffer_with_encoder(&mut encoder, capacity);
        self.gpu.queue.submit(Some(encoder.finish()));
        res
    }
    pub fn create_buffer_with_encoder(&mut self, encoder: &mut wgpu::CommandEncoder, capacity: Option<u64>) -> SubBufferId {
        let new_sub_buffer = Some(SubBuffer { offset_bytes: self.total_capacity, size_bytes: 0, capacity_bytes: 0 });
        let id = if let Some(id) = self.free_ids.pop() {
            self.sub_buffers[id] = new_sub_buffer;
            id
        } else {
            let id = self.sub_buffers.len();
            self.sub_buffers.push(new_sub_buffer);
            id
        };
        // Buffers have to have some capacity, otherwise their order is indeterminable (if two buffers next to each other
        // have capacity=0 then their offsets will both be 0)
        self.change_buffer_capacity(encoder, id, capacity.unwrap_or(4));
        id
    }
    pub fn remove_buffer(&mut self, id: SubBufferId) -> Result<(), MultiBufferError> {
        let mut encoder =
            self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("MultiBuffer.remove_buffer") });
        let res = self.remove_buffer_with_encoder(&mut encoder, id);
        self.gpu.queue.submit(Some(encoder.finish()));
        res
    }
    pub fn remove_buffer_with_encoder(&mut self, encoder: &mut wgpu::CommandEncoder, id: SubBufferId) -> Result<(), MultiBufferError> {
        if self.buffer_exists(id) {
            self.change_buffer_capacity(encoder, id, 0);
            self.sub_buffers[id] = None;
            self.free_ids.push(id);
            Ok(())
        } else {
            Err(MultiBufferError::NoSuchSubBuffer(id))
        }
    }

    pub fn buffer_exists(&self, id: SubBufferId) -> bool {
        matches!(self.sub_buffers.get(id), Some(Some(_)))
    }

    pub fn resize_buffer(&mut self, id: SubBufferId, len: u64) -> Result<(), MultiBufferError> {
        let mut encoder =
            self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("MultiBuffer.resize_buffer") });
        let res = self.resize_buffer_with_encoder(&mut encoder, id, len);
        self.gpu.queue.submit(Some(encoder.finish()));
        res
    }

    pub fn resize_buffer_with_encoder(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        id: SubBufferId,
        len: u64,
    ) -> Result<(), MultiBufferError> {
        let mut change_capacity = None;
        if let Some(Some(buf)) = self.sub_buffers.get_mut(id) {
            buf.size_bytes = len;
            let cap = match self.size_strategy {
                MultiBufferSizeStrategy::Exact => len,
                MultiBufferSizeStrategy::Pow2 => 2u64.pow((len as f32).log2().ceil() as u32),
            };
            if cap != buf.capacity_bytes {
                change_capacity = Some(cap);
            }
        } else {
            return Err(MultiBufferError::NoSuchSubBuffer(id));
        }
        if let Some(new_capacity) = change_capacity {
            self.change_buffer_capacity(encoder, id, new_capacity);
        }
        Ok(())
    }

    pub fn write(&self, id: SubBufferId, offset: u64, data: &[u8]) -> Result<(), MultiBufferError> {
        if let Some(Some(buf)) = &self.sub_buffers.get(id) {
            if offset + (data.len() as u64) > buf.size_bytes {
                return Err(MultiBufferError::WriteOfRange { offset, data_len: data.len() as u64, buffer_len: buf.size_bytes });
            }
            self.gpu.queue.write_buffer(&self.buffer, buf.offset_bytes + offset, data);
            Ok(())
        } else {
            Err(MultiBufferError::NoSuchSubBuffer(id))
        }
    }
    pub fn buffer_len(&self, id: SubBufferId) -> Result<u64, MultiBufferError> {
        if let Some(Some(buf)) = &self.sub_buffers.get(id) {
            Ok(buf.size_bytes)
        } else {
            Err(MultiBufferError::NoSuchSubBuffer(id))
        }
    }
    pub fn buffer_layout(&self, id: SubBufferId) -> Result<SubBuffer, MultiBufferError> {
        if let Some(Some(buf)) = &self.sub_buffers.get(id) {
            Ok(*buf)
        } else {
            Err(MultiBufferError::NoSuchSubBuffer(id))
        }
    }
    fn change_buffer_capacity(&mut self, encoder: &mut wgpu::CommandEncoder, id: SubBufferId, capacity: u64) {
        // Wgpu requires copy_buffer_to_buffer to align to COPY_BUFFER_ALIGNMENT, so each of our sub-buffers need to align to that too
        assert_eq!(capacity % wgpu::COPY_BUFFER_ALIGNMENT, 0);
        let buf = self.sub_buffers[id].unwrap();
        let capacity_change = capacity as i64 - buf.capacity_bytes as i64;
        let new_total_capacity = (self.total_capacity as i64 + capacity_change) as u64;
        if capacity_change > 0 {
            MULTI_BUFFERS_TOTAL_SIZE.fetch_add(capacity_change as usize, Ordering::SeqCst);
        } else {
            MULTI_BUFFERS_TOTAL_SIZE.fetch_sub((-capacity_change) as usize, Ordering::SeqCst);
        }
        let new_buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            usage: self.usage,
            size: new_total_capacity,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, (buf.offset_bytes + buf.capacity_bytes).min(new_total_capacity));
        self.sub_buffers[id].as_mut().unwrap().capacity_bytes = capacity;
        if let Some(next_id) = self.get_next_buffer(buf.offset_bytes) {
            let next = self.sub_buffers[next_id].unwrap();
            encoder.copy_buffer_to_buffer(
                &self.buffer,
                next.offset_bytes,
                &new_buffer,
                buf.offset_bytes + capacity,
                self.total_capacity - next.offset_bytes,
            );
            for b in self.sub_buffers.iter_mut().flatten() {
                if b.offset_bytes > buf.offset_bytes {
                    b.offset_bytes = (b.offset_bytes as i64 + capacity_change) as u64;
                }
            }
        }
        self.buffer = new_buffer;
        self.total_capacity = new_total_capacity;
    }
    fn get_next_buffer(&self, offset: u64) -> Option<SubBufferId> {
        self.sub_buffers
            .iter()
            .enumerate()
            .filter(|(_, buf)| if let Some(buf) = buf { buf.offset_bytes > offset } else { false })
            .min_by_key(|(_, buf)| buf.as_ref().unwrap().offset_bytes)
            .map(|(index, _)| index)
    }
    #[cfg(test)]
    async fn read_all(&self) -> Vec<u8> {
        self.read_range(0, self.total_capacity).await
    }
    #[cfg(test)]
    async fn read_range(&self, offset: u64, size: u64) -> Vec<u8> {
        if size == 0 {
            return Vec::new();
        }
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let staging_buffer = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&self.buffer, offset, &staging_buffer, 0, size);

        self.gpu.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });
        if !self.gpu.will_be_polled {
            self.gpu.device.poll(wgpu::Maintain::Wait);
        }
        rx.await.unwrap().unwrap();
        let range = buffer_slice.get_mapped_range();
        let data = range.to_vec();
        drop(range);
        staging_buffer.unmap();
        data
    }
}
impl Drop for MultiBuffer {
    fn drop(&mut self) {
        MULTI_BUFFERS_TOTAL_SIZE.fetch_sub(self.total_capacity as usize, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SubBuffer {
    pub offset_bytes: u64,
    pub size_bytes: u64,
    pub capacity_bytes: u64,
}

pub struct TypedMultiBuffer<T: bytemuck::Pod> {
    buffer: MultiBuffer,
    item_size: u64,
    _type: PhantomData<T>,
}
impl<T: bytemuck::Pod> TypedMultiBuffer<T> {
    pub fn new(gpu: Arc<Gpu>, label: &str, usage: wgpu::BufferUsages, size_strategy: MultiBufferSizeStrategy) -> Self {
        Self { buffer: MultiBuffer::new(gpu, label, usage, size_strategy), item_size: std::mem::size_of::<T>() as u64, _type: PhantomData }
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.buffer()
    }
    pub fn total_capacity_in_bytes(&self) -> u64 {
        self.buffer.total_capacity_in_bytes()
    }
    pub fn total_len(&self) -> u64 {
        self.buffer.total_capacity_in_bytes() / self.item_size
    }
    pub fn create_buffer(&mut self, capacity: Option<u64>) -> SubBufferId {
        self.buffer.create_buffer(capacity)
    }
    pub fn create_buffer_with_encoder(&mut self, encoder: &mut wgpu::CommandEncoder, capacity: Option<u64>) -> SubBufferId {
        self.buffer.create_buffer_with_encoder(encoder, capacity)
    }
    pub fn remove_buffer(&mut self, id: SubBufferId) -> Result<(), MultiBufferError> {
        self.buffer.remove_buffer(id)
    }
    pub fn remove_buffer_with_encoder(&mut self, encoder: &mut wgpu::CommandEncoder, id: SubBufferId) -> Result<(), MultiBufferError> {
        self.buffer.remove_buffer_with_encoder(encoder, id)
    }
    pub fn buffer_exists(&self, id: SubBufferId) -> bool {
        self.buffer.buffer_exists(id)
    }
    pub fn resize_buffer(&mut self, id: SubBufferId, len: u64) -> Result<(), MultiBufferError> {
        self.buffer.resize_buffer(id, len * self.item_size)
    }
    pub fn resize_buffer_with_encoder(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        id: SubBufferId,
        len: u64,
    ) -> Result<(), MultiBufferError> {
        self.buffer.resize_buffer_with_encoder(encoder, id, len * self.item_size)
    }
    pub fn buffer_offset(&self, id: SubBufferId) -> Result<u64, MultiBufferError> {
        Ok(self.buffer.buffer_layout(id)?.offset_bytes / self.item_size)
    }
    pub fn write(&self, id: SubBufferId, offset: u64, data: &[T]) -> Result<(), MultiBufferError> {
        self.buffer.write(id, offset * self.item_size, bytemuck::cast_slice(data))
    }
    pub fn buffer_len(&self, id: SubBufferId) -> Result<u64, MultiBufferError> {
        self.buffer.buffer_len(id).map(|len| len / self.item_size)
    }
    pub fn push(&mut self, id: SubBufferId, value: T) -> Result<(), MultiBufferError> {
        let len = self.buffer.buffer_len(id)?;
        self.buffer.resize_buffer(id, len + self.item_size)?;
        self.buffer.write(id, len * self.item_size, bytemuck::cast_slice(&[value]))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_multi_buffer() {
        let gpu = Arc::new(Gpu::new(None).await);
        let mut buf =
            MultiBuffer::new(gpu, "test", wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, MultiBufferSizeStrategy::Pow2);

        let a = buf.create_buffer(None);
        buf.resize_buffer(a, 4).unwrap();
        buf.write(a, 0, &[1, 1, 1, 1]).unwrap();
        assert_eq!(buf.read_all().await, &[1, 1, 1, 1]);
        buf.resize_buffer(a, 5).unwrap();
        assert_eq!(buf.read_all().await, &[1, 1, 1, 1, 0, 0, 0, 0]);
        buf.resize_buffer(a, 8).unwrap();
        buf.write(a, 4, &[2, 2, 2, 2]).unwrap();
        assert_eq!(buf.read_all().await, &[1, 1, 1, 1, 2, 2, 2, 2]);

        let b = buf.create_buffer(None);
        buf.resize_buffer(b, 4).unwrap();
        assert_eq!(buf.read_all().await, &[1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0]);
        buf.write(b, 0, &[3, 3, 3, 3]).unwrap();
        assert_eq!(buf.read_all().await, &[1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
        buf.resize_buffer(a, 9).unwrap();
        assert_eq!(buf.read_all().await, &[1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3]);
        buf.remove_buffer(a).unwrap();
        assert_eq!(buf.read_all().await, &[3, 3, 3, 3]);

        buf.remove_buffer(b).unwrap();
        assert_eq!(buf.read_all().await, &[] as &[u8]);
    }

    #[tokio::test]
    async fn test_multi_buffer2() {
        let gpu = Arc::new(Gpu::new(None).await);
        let mut buf =
            MultiBuffer::new(gpu, "test", wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, MultiBufferSizeStrategy::Pow2);

        let a = buf.create_buffer(None);
        let b = buf.create_buffer(None);
        buf.resize_buffer(b, 4).unwrap();
        buf.resize_buffer(a, 4).unwrap();
        assert_eq!(buf.buffer_layout(b).unwrap().offset_bytes, 4);
    }

    #[tokio::test]
    async fn test_multi_buffer_reuse_id() {
        let gpu = Arc::new(Gpu::new(None).await);
        let mut buf =
            MultiBuffer::new(gpu, "test", wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, MultiBufferSizeStrategy::Pow2);

        let a = buf.create_buffer(None);
        buf.remove_buffer(a).unwrap();
        buf.create_buffer(None);
    }

    #[tokio::test]
    async fn test_multi_buffer_shrink() {
        let gpu = Arc::new(Gpu::new(None).await);
        let mut buf =
            MultiBuffer::new(gpu, "test", wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST, MultiBufferSizeStrategy::Pow2);

        let a = buf.create_buffer(None);
        buf.resize_buffer(a, 20).unwrap();
        buf.resize_buffer(a, 4).unwrap();
        assert_eq!(buf.total_capacity_in_bytes(), 4);
    }
}
