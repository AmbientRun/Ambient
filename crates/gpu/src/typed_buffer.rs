use std::{
    any::type_name,
    marker::PhantomData,
    mem::{self, size_of},
    ops::{Bound, DerefMut, Range, RangeBounds},
    sync::atomic::{AtomicUsize, Ordering},
};

use ambient_std::asset_cache::AssetCache;
use futures::Future;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferAddress, BufferAsyncError, BufferDescriptor, CommandEncoder,
};

use super::gpu::Gpu;

static UNTYPED_BUFFERS_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

pub struct UntypedBuffer {
    label: String,
    pub buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    capacity: u64,
    size: u64,
}

impl std::fmt::Debug for UntypedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UntypedBuffer")
            .field("capacity", &self.capacity)
            .field("length", &self.size)
            .finish()
    }
}

impl UntypedBuffer {
    pub fn new(
        gpu: &Gpu,
        label: &str,
        capacity: u64,
        size: u64,
        usage: wgpu::BufferUsages,
    ) -> Self {
        UNTYPED_BUFFERS_TOTAL_SIZE.fetch_add((capacity) as usize, Ordering::SeqCst);
        Self {
            buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                usage,
                size: capacity,
                mapped_at_creation: false,
            }),
            label: label.to_string(),
            usage,
            capacity,
            size,
        }
    }

    pub fn new_init(gpu: &Gpu, label: &str, usage: wgpu::BufferUsages, data: &[u8]) -> Self {
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
            capacity: data.len() as u64,
            size: data.len() as u64,
        }
    }

    pub fn total_bytes_used() -> usize {
        UNTYPED_BUFFERS_TOTAL_SIZE.load(Ordering::SeqCst)
    }

    /// Returns true if the capacity changed
    /// Setting retain_content to false will make the buffer zero out when a new buffer is created
    pub fn resize(&mut self, gpu: &Gpu, new_size: u64, retain_content: bool) -> bool {
        self.size = new_size;
        if self.capacity < new_size {
            let cap = new_size.next_power_of_two();
            self.change_capacity(gpu, cap, retain_content);
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Size in bytes
    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn write(&self, gpu: &Gpu, offset: u64, data: &[u8]) {
        assert!(
            (offset + data.len() as u64) < self.capacity,
            "Out of bounds write"
        );
        gpu.queue.write_buffer(&self.buffer, offset, data);
    }

    fn change_capacity(&mut self, gpu: &Gpu, new_capacity: usize, retain_content: bool) {
        if new_capacity > self.capacity {
            UNTYPED_BUFFERS_TOTAL_SIZE
                .fetch_add((new_capacity - self.capacity) as usize, Ordering::SeqCst);
        } else {
            UNTYPED_BUFFERS_TOTAL_SIZE
                .fetch_sub((self.capacity - new_capacity) as usize, Ordering::SeqCst);
        }
        let new_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            usage: self.usage,
            size: new_capacity,
            mapped_at_creation: false,
        });

        if retain_content {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("retain_content"),
                });

            encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, self.size);
            gpu.queue.submit(Some(encoder.finish()));
        }

        self.buffer = new_buffer;
        self.capacity = new_capacity;
    }

    pub async fn read_direct(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<BufferAddress>,
    ) -> Result<Vec<u8>, BufferAsyncError> {
        let read = Self::read_buf(&self.buffer, bounds);

        if !gpu.will_be_polled {
            gpu.device.poll(wgpu::Maintain::Wait);
        }

        read.await
    }

    pub fn read_staging<'a>(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<BufferAddress>,
    ) -> impl Future<Output = Result<Vec<u8>, BufferAsyncError>> + 'a {
        let start = match bounds.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0,
        };

        let end = match bounds.end_bound() {
            Bound::Included(v) => *v + 1,
            Bound::Excluded(v) => *v,
            Bound::Unbounded => self.size,
        };

        let size = end - start;
        if size == 0 {
            panic!("Cannot read 0 bytes from a buffer");
        }

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        tracing::debug!("Creating staging buffer of {size}");
        let staging_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        tracing::debug!("Copying {size} bytes to staging buffer");
        encoder.copy_buffer_to_buffer(&self.buffer, start, &staging_buffer, 0, size);
        gpu.queue.submit(Some(encoder.finish()));

        async move { Self::read_buf(&staging_buffer, ..).await }
    }

    fn read_buf<'a>(
        // gpu: &'a Gpu,
        buf: &'a wgpu::Buffer,
        range: impl RangeBounds<BufferAddress>,
    ) -> impl Future<Output = Result<Vec<u8>, BufferAsyncError>> + 'a {
        let slice = buf.slice(range);
        let (tx, value) = tokio::sync::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });

        // if !gpu.will_be_polled {
        //     gpu.device.poll(wgpu::Maintain::Wait);
        // }

        async move {
            value.await.unwrap()?;
            let range = slice.get_mapped_range();
            let data = range.to_vec();
            drop(range);
            buf.unmap();
            Ok(data)
        }
    }
}

impl Drop for UntypedBuffer {
    fn drop(&mut self) {
        UNTYPED_BUFFERS_TOTAL_SIZE.fetch_sub(self.size() as usize, Ordering::SeqCst);
    }
}

pub struct TypedBuffer<T: bytemuck::Pod> {
    label: String,
    buffer: wgpu::Buffer,
    len: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T: bytemuck::Pod + std::fmt::Debug> std::fmt::Debug for TypedBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedBuffer")
            .field("type", &type_name::<T>())
            .field("capacity", &self.capacity)
            .field("len", &self.len)
            .finish()
    }
}

impl<T: bytemuck::Pod> TypedBuffer<T> {
    pub fn new(
        gpu: &Gpu,
        label: impl Into<String>,
        capacity: usize,
        len: usize,
        usage: wgpu::BufferUsages,
    ) -> Self {
        let label = label.into();

        let buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some(&label),
            usage,
            size: (mem::size_of::<T>() as u64 * len as u64),
            mapped_at_creation: false,
        });

        Self {
            label,
            buffer,
            len,
            capacity: len,
            _marker: PhantomData,
        }
    }

    pub fn new_init(
        gpu: &Gpu,
        label: impl Into<String>,
        usage: wgpu::BufferUsages,
        data: &[T],
    ) -> Self {
        let label = label.into();

        let buffer = gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&label),
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Self {
            label,
            buffer,
            len: data.len() as _,
            capacity: data.len() as _,
            _marker: PhantomData,
        }
    }

    /// Returns true if the capacity changed
    pub fn resize(&mut self, gpu: &Gpu, new_len: u64, retain_content: bool) -> bool {
        self.buffer.resize(gpu, new_len, retain_content)
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Allocated buffer size in bytes
    pub fn byte_size(&self) -> u64 {
        self.capacity as u64 * mem::size_of::<T>() as u64
    }

    #[inline]
    pub const fn item_size(&self) -> u64 {
        mem::size_of::<T>() as u64
    }

    pub fn write(&self, gpu: &Gpu, index: usize, data: &[T]) {
        assert!(data.len() + index <= self.capacity);
        gpu.queue.write_buffer(
            &self.buffer,
            index as u64 * size_of::<T>() as u64,
            bytemuck::cast_slice(data),
        );
    }

    /// Reads a range from the buffer. The range is defined in items; i.e. 1..3 means read item 1 through 3 (not bytes).
    pub async fn read_direct(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<usize>,
    ) -> Result<Vec<T>, BufferAsyncError> {
        // Convert the bounds to byte offsets
        let start = match bounds.start_bound() {
            Bound::Included(&v) => v as u64,
            Bound::Excluded(&v) => v as u64 + 1,
            Bound::Unbounded => 0,
        } * size_of::<T>() as u64;

        let end = match bounds.end_bound() {
            Bound::Included(&v) => (v as u64 + 1),
            Bound::Excluded(&v) => v as u64,
            Bound::Unbounded => self.len as u64,
        } * size_of::<T>() as u64;

        let read = Self::read_buf(&self.buffer, start..end);

        if !gpu.will_be_polled {
            gpu.device.poll(wgpu::Maintain::Wait);
        }

        read.await
    }

    /// Reads a range from the buffer. The range is defined in items; i.e. 1..3 means read item 1 through 3 (not bytes).
    pub fn read_staging<'a>(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<u64>,
    ) -> impl Future<Output = Result<Vec<T>, BufferAsyncError>> {
        // Convert the bounds to byte offsets
        let start = match bounds.start_bound() {
            Bound::Included(&v) => v as u64,
            Bound::Excluded(&v) => v as u64 + 1,
            Bound::Unbounded => 0,
        } * size_of::<T>() as u64;

        let end = match bounds.end_bound() {
            Bound::Included(&v) => (v as u64 + 1),
            Bound::Excluded(&v) => v as u64,
            Bound::Unbounded => self.len as u64,
        } * size_of::<T>() as u64;

        let size = end - start;
        if size == 0 {
            panic!("Cannot read 0 bytes from a buffer");
        }

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        tracing::debug!("Creating staging buffer of {size}");
        let staging_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        tracing::debug!("Copying {size} bytes to staging buffer");
        encoder.copy_buffer_to_buffer(&self.buffer, start, &staging_buffer, 0, size);
        gpu.queue.submit(Some(encoder.finish()));

        async move { Self::read_buf(&staging_buffer, ..).await }
    }

    fn read_buf<'a>(
        // gpu: &'a Gpu,
        buf: &'a wgpu::Buffer,
        range: impl RangeBounds<BufferAddress>,
    ) -> impl Future<Output = Result<Vec<T>, BufferAsyncError>> + 'a {
        let slice = buf.slice(range);
        let (tx, value) = tokio::sync::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });

        // if !gpu.will_be_polled {
        //     gpu.device.poll(wgpu::Maintain::Wait);
        // }

        async move {
            value.await.unwrap()?;
            let range = slice.get_mapped_range();
            let data: Vec<T> = bytemuck::cast_vec(range.to_vec());

            drop(range);

            buf.unmap();

            Ok(data)
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn push(&mut self, gpu: &Gpu, val: T, mut on_resize: impl FnMut(&Self)) {
        if self.len() < self.capacity() {
            self.write(gpu, self.len, &[val]);
            self.len += 1;
        } else {
            let new_cap = self.capacity * 2;
            self.change_capacity(gpu, new_cap, true);
            self.write(gpu, self.len, &[val]);
            self.len += 1;
            on_resize(self)
        }
    }

    fn change_capacity(&mut self, gpu: &Gpu, new_capacity: usize, retain_content: bool) {
        if new_capacity > self.capacity {
            UNTYPED_BUFFERS_TOTAL_SIZE.fetch_add(
                (new_capacity - self.capacity) * size_of::<T>() as usize,
                Ordering::SeqCst,
            );
        } else {
            UNTYPED_BUFFERS_TOTAL_SIZE.fetch_sub(
                (self.capacity - new_capacity) * size_of::<T>() as usize,
                Ordering::SeqCst,
            );
        }

        let new_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            usage: self.buffer.usage(),
            size: new_capacity as u64 * size_of::<T>() as u64,
            mapped_at_creation: false,
        });

        if retain_content {
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("retain_content"),
                });

            encoder.copy_buffer_to_buffer(
                &self.buffer,
                0,
                &new_buffer,
                0,
                self.len as u64 * size_of::<T>() as u64,
            );
            gpu.queue.submit(Some(encoder.finish()));
        }

        self.buffer = new_buffer;
        self.capacity = new_capacity;
    }

    pub fn fill(&mut self, gpu: &Gpu, data: &[T], mut on_resize: impl FnMut(&Self)) {
        if self.resize(gpu, data.len() as u64, true) {
            on_resize(self);
        }

        self.write(gpu, 0, data)
    }
}

impl<T: bytemuck::Pod> std::ops::Deref for TypedBuffer<T> {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
