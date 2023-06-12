use std::{
    marker::PhantomData,
    ops::{DerefMut, RangeBounds},
    sync::atomic::{AtomicUsize, Ordering},
};

use wgpu::{util::DeviceExt, BufferAddress, BufferAsyncError};

use super::gpu::Gpu;

static UNTYPED_BUFFERS_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct UntypedBuffer {
    label: String,
    pub buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    capacity: u64,
    length: u64,
    item_size: u64,
}

impl UntypedBuffer {
    pub fn new(
        gpu: &Gpu,
        label: &str,
        capacity: u64,
        length: u64,
        usage: wgpu::BufferUsages,
        item_size: u64,
    ) -> Self {
        UNTYPED_BUFFERS_TOTAL_SIZE.fetch_add((capacity * item_size) as usize, Ordering::SeqCst);
        Self {
            buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                usage,
                size: capacity * item_size,
                mapped_at_creation: false,
            }),
            label: label.to_string(),
            usage,
            capacity,
            length,
            item_size,
        }
    }

    pub fn new_init(
        gpu: &Gpu,
        label: &str,
        usage: wgpu::BufferUsages,
        data: &[u8],
        item_size: u64,
    ) -> Self {
        UNTYPED_BUFFERS_TOTAL_SIZE.fetch_add(data.len(), Ordering::SeqCst);
        Self {
            buffer: gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    usage,
                    contents: data,
                }),
            label: label.to_string(),
            usage,
            capacity: data.len() as u64 / item_size,
            length: data.len() as u64 / item_size,
            item_size,
        }
    }

    pub fn total_bytes_used() -> usize {
        UNTYPED_BUFFERS_TOTAL_SIZE.load(Ordering::SeqCst)
    }

    /// Returns true if the capacity changed
    /// Setting retain_content to false will make the buffer zero out when a new buffer is created
    pub fn resize(&mut self, gpu: &Gpu, new_len: u64, retain_content: bool) -> bool {
        self.length = new_len;
        if self.capacity < new_len {
            let cap = 2u64.pow((new_len as f32).log2().ceil() as u32);
            self.change_capacity(gpu, cap, retain_content);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> u64 {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn item_size(&self) -> u64 {
        self.item_size
    }

    /// Size in bytes
    pub fn size(&self) -> u64 {
        self.length * self.item_size
    }

    pub fn write(&self, gpu: &Gpu, index: u64, data: &[u8]) {
        gpu.queue
            .write_buffer(&self.buffer, index * self.item_size, data);
    }

    fn change_capacity(&mut self, gpu: &Gpu, new_capacity: u64, retain_content: bool) {
        if new_capacity > self.capacity {
            UNTYPED_BUFFERS_TOTAL_SIZE.fetch_add(
                ((new_capacity - self.capacity) * self.item_size) as usize,
                Ordering::SeqCst,
            );
        } else {
            UNTYPED_BUFFERS_TOTAL_SIZE.fetch_sub(
                ((self.capacity - new_capacity) * self.item_size) as usize,
                Ordering::SeqCst,
            );
        }
        let new_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            usage: self.usage,
            size: new_capacity * self.item_size,
            mapped_at_creation: false,
        });
        if retain_content {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_buffer_to_buffer(
                &self.buffer,
                0,
                &new_buffer,
                0,
                self.capacity * self.item_size,
            );
            gpu.queue.submit(Some(encoder.finish()));
        }
        self.buffer = new_buffer;
        self.capacity = new_capacity;
    }

    /// If use_staging is true it will create a temporary staging buffer internally, copy the data over, and then read from that
    pub async fn read(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<BufferAddress>,
        use_staging: bool,
    ) -> Result<Vec<u8>, BufferAsyncError> {
        if use_staging {
            let start = match bounds.start_bound() {
                std::ops::Bound::Included(v) => *v,
                std::ops::Bound::Excluded(v) => *v + 1,
                std::ops::Bound::Unbounded => 0,
            };
            let end = match bounds.end_bound() {
                std::ops::Bound::Included(v) => *v + 1,
                std::ops::Bound::Excluded(v) => *v,
                std::ops::Bound::Unbounded => self.length,
            };
            let size = end - start;
            if size == 0 {
                return Ok(Vec::new());
            }

            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let staging_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            encoder.copy_buffer_to_buffer(&self.buffer, start, &staging_buffer, 0, size);
            gpu.queue.submit(Some(encoder.finish()));
            Self::read_buf(gpu, &staging_buffer, ..).await
        } else {
            Self::read_buf(gpu, &self.buffer, bounds).await
        }
    }
    async fn read_buf(
        gpu: &Gpu,
        buf: &wgpu::Buffer,
        range: impl RangeBounds<BufferAddress>,
    ) -> Result<Vec<u8>, BufferAsyncError> {
        let slice = buf.slice(range);
        let (tx, value) = tokio::sync::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });
        if !gpu.will_be_polled {
            gpu.device.poll(wgpu::Maintain::Wait);
        }
        value.await.unwrap()?;
        let range = slice.get_mapped_range();
        let data = range.to_vec();
        drop(range);
        buf.unmap();
        Ok(data)
    }
}
impl Drop for UntypedBuffer {
    fn drop(&mut self) {
        UNTYPED_BUFFERS_TOTAL_SIZE.fetch_sub(self.size() as usize, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct TypedBuffer<T: bytemuck::Pod> {
    buffer: UntypedBuffer,
    _type: PhantomData<T>,
}

impl<T: bytemuck::Pod> TypedBuffer<T> {
    pub fn new(
        gpu: &Gpu,
        label: &str,
        capacity: u64,
        length: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        Self {
            buffer: UntypedBuffer::new(
                gpu,
                label,
                capacity,
                length,
                usage,
                std::mem::size_of::<T>() as u64,
            ),
            _type: PhantomData,
        }
    }

    pub fn new_init(gpu: &Gpu, label: &str, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        Self {
            buffer: UntypedBuffer::new_init(
                gpu,
                label,
                usage,
                bytemuck::cast_slice(data),
                std::mem::size_of::<T>() as u64,
            ),
            _type: PhantomData,
        }
    }

    /// Returns true if the capacity changed
    pub fn resize(&mut self, gpu: &Gpu, new_len: u64, retain_content: bool) -> bool {
        self.buffer.resize(gpu, new_len, retain_content)
    }

    pub fn len(&self) -> u64 {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Size in bytes
    pub fn byte_size(&self) -> u64 {
        self.buffer.size()
    }

    pub fn item_size(&self) -> u64 {
        self.buffer.item_size()
    }

    pub fn write(&self, gpu: &Gpu, index: u64, data: &[T]) {
        assert!(data.len() as u64 + index <= self.capacity);
        self.buffer.write(gpu, index, bytemuck::cast_slice(data));
    }

    /// Reads a range from the buffer. The range is defined in items; i.e. 1..3 means read item 1 through 3 (not bytes).
    pub async fn read(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<u64>,
        use_staging: bool,
    ) -> Result<Vec<T>, BufferAsyncError> {
        let start = match bounds.start_bound() {
            std::ops::Bound::Included(v) => v * self.item_size,
            std::ops::Bound::Excluded(v) => (v + 1) * self.item_size,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match bounds.end_bound() {
            std::ops::Bound::Included(v) => (v + 1) * self.item_size,
            std::ops::Bound::Excluded(v) => v * self.item_size,
            std::ops::Bound::Unbounded => self.length * self.item_size,
        };

        let data = self.buffer.read(gpu, start..end, use_staging).await?;
        Ok(bytemuck::cast_slice(&data).to_vec())
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer.buffer
    }

    pub fn push(&mut self, gpu: &Gpu, val: T, mut on_resize: impl FnMut(&Self)) {
        if self.length < self.capacity {
            self.write(gpu, self.length, &[val]);
            self.length += 1;
        } else {
            let new_cap = self.capacity * 2;
            self.change_capacity(gpu, new_cap, true);
            self.write(gpu, self.length, &[val]);
            self.length += 1;
            on_resize(self)
        }
    }

    pub fn fill(&mut self, gpu: &Gpu, data: &[T], mut on_resize: impl FnMut(&Self)) {
        if self.resize(gpu, data.len() as u64, true) {
            on_resize(self);
        }

        self.write(gpu, 0, data)
    }
}

impl<T: bytemuck::Pod> std::ops::Deref for TypedBuffer<T> {
    type Target = UntypedBuffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T: bytemuck::Pod> DerefMut for TypedBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
