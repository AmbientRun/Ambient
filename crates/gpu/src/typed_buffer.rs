use std::{
    any::type_name,
    marker::PhantomData,
    mem::{self, size_of},
    num::NonZeroU64,
    ops::{Bound, Range, RangeBounds},
    sync::atomic::{AtomicU64, Ordering},
};

use bytemuck::Pod;
use futures::Future;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BufferAddress, BufferAsyncError, BufferDescriptor,
};

use super::gpu::Gpu;

static TOTAL_ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

pub struct TypedBuffer<T> {
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

impl<T> Drop for TypedBuffer<T> {
    fn drop(&mut self) {
        TOTAL_ALLOCATED_BYTES.fetch_sub(self.byte_capacity(), Ordering::SeqCst);
    }
}

impl<T> TypedBuffer<T> {
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
            size: (mem::size_of::<T>() as u64 * capacity as u64),
            mapped_at_creation: false,
        });

        TOTAL_ALLOCATED_BYTES
            .fetch_add((capacity) as u64 * size_of::<T>() as u64, Ordering::SeqCst);

        Self {
            label,
            buffer,
            len,
            capacity: len,
            _marker: PhantomData,
        }
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
    pub fn byte_capacity(&self) -> BufferAddress {
        self.capacity as u64 * mem::size_of::<T>() as u64
    }

    /// Initialized buffer size in bytes
    pub fn byte_len(&self) -> BufferAddress {
        self.len as u64 * mem::size_of::<T>() as u64
    }

    #[inline]
    pub const fn item_size(&self) -> BufferAddress {
        mem::size_of::<T>() as u64
    }

    /// Converts an index range into a byte offset range
    pub fn to_byte_offsets(&self, bounds: impl RangeBounds<usize>) -> Range<BufferAddress> {
        let start = match bounds.start_bound() {
            Bound::Included(&v) => v,
            Bound::Excluded(&v) => v + 1,
            Bound::Unbounded => 0,
        } as u64
            * size_of::<T>() as u64;

        let end = match bounds.end_bound() {
            Bound::Included(&v) => v + 1,
            Bound::Excluded(&v) => v,
            Bound::Unbounded => self.len,
        } as u64
            * size_of::<T>() as u64;

        start..end
    }

    #[inline]
    /// Returns the underlying wgpu::Buffer
    ///
    /// **Note**: The same instance will not always be returned due to reallocation
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn as_binding(&self) -> BindingResource {
        BindingResource::Buffer(wgpu::BufferBinding {
            buffer: self.buffer(),
            offset: 0,
            size: Some(NonZeroU64::new(self.byte_len()).expect("buffer size is 0")),
        })
    }

    fn change_capacity(&mut self, gpu: &Gpu, new_capacity: usize, retain_content: bool) {
        if new_capacity > self.capacity {
            TOTAL_ALLOCATED_BYTES.fetch_add(
                (new_capacity - self.capacity) as u64 * size_of::<T>() as u64,
                Ordering::SeqCst,
            );
        } else {
            TOTAL_ALLOCATED_BYTES.fetch_sub(
                (self.capacity - new_capacity) as u64 * size_of::<T>() as u64,
                Ordering::SeqCst,
            );
        }

        let new_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            usage: self.buffer.usage(),
            size: new_capacity as u64 * size_of::<T>() as u64,
            mapped_at_creation: false,
        });

        if retain_content && self.len > 0 {
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
}

impl<T: Pod> TypedBuffer<T> {
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

        TOTAL_ALLOCATED_BYTES
            .fetch_add(data.len() as u64 * size_of::<T>() as u64, Ordering::SeqCst);

        Self {
            label,
            buffer,
            len: data.len() as _,
            capacity: data.len() as _,
            _marker: PhantomData,
        }
    }

    /// Sets the length of the *initialized* region of the buffer.
    ///
    /// Will increase capacity as appropriate.
    ///
    /// Returns true if the buffer was resized
    pub fn set_len(&mut self, gpu: &Gpu, len: usize) -> bool {
        self.set_len_inner(gpu, len, true)
    }

    /// Same as [`Self::set_len`] but discards the contents when resized.
    ///
    /// Use this if you immediately fill the buffer
    pub fn set_len_discard(&mut self, gpu: &Gpu, len: usize) -> bool {
        self.set_len_inner(gpu, len, false)
    }

    fn set_len_inner(&mut self, gpu: &Gpu, len: usize, retain: bool) -> bool {
        if len > self.capacity {
            self.change_capacity(gpu, len.next_power_of_two(), retain);
            self.len = len;
            true
        } else {
            self.len = len;
            false
        }
    }

    pub fn write(&self, gpu: &Gpu, index: usize, data: &[T]) {
        assert!(
            data.len() + index <= self.len,
            "Writing outside initialized bounds of buffer"
        );

        gpu.queue.write_buffer(
            &self.buffer,
            index as u64 * size_of::<T>() as u64,
            bytemuck::cast_slice(data),
        );
    }

    /// Reads a range from the buffer. The range is defined in items; i.e. 1..3 means read item 1 through 3 (not bytes).
    pub async fn read(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<usize>,
    ) -> Result<Vec<T>, BufferAsyncError> {
        // Convert the bounds to byte offsets

        let bounds = self.to_byte_offsets(bounds);
        let (tx, rx) = tokio::sync::oneshot::channel();
        let slice = self.buffer().slice(bounds);

        slice.map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });

        if !gpu.will_be_polled {
            eprintln!("Polling gpu");
            gpu.device.poll(wgpu::Maintain::Wait);
        }

        rx.await.unwrap()?;

        let data: Vec<T> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();

        self.buffer().unmap();

        Ok(data)
    }

    /// Reads a range from the buffer. The range is defined in items; i.e. 1..3 means read item 1 through 3 (not bytes).
    pub fn read_staging(
        &self,
        gpu: &Gpu,
        bounds: impl RangeBounds<usize>,
    ) -> impl Future<Output = Result<Vec<T>, BufferAsyncError>> {
        // Convert the bounds to byte offsets
        let bounds = self.to_byte_offsets(bounds);

        let size = bounds.end - bounds.start;

        if size == 0 {
            panic!("Cannot read 0 bytes from a buffer");
        }

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        tracing::debug!("Creating staging buffer of {size}");

        let sb = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        tracing::debug!("Copying {size} bytes to staging buffer");
        encoder.copy_buffer_to_buffer(&self.buffer, bounds.start, &sb, 0, size);
        gpu.queue.submit(Some(encoder.finish()));

        let (tx, rx) = tokio::sync::oneshot::channel();
        sb.slice(..).map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });

        if !gpu.will_be_polled {
            eprintln!("Polling gpu");
            gpu.device.poll(wgpu::Maintain::Wait);
        }

        async move {
            rx.await.unwrap()?;

            let data: Vec<T> = bytemuck::cast_slice(&sb.slice(..).get_mapped_range()).to_vec();

            sb.unmap();

            Ok(data)
        }
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

    pub fn fill(&mut self, gpu: &Gpu, data: &[T], mut on_resize: impl FnMut(&Self)) {
        if self.set_len_discard(gpu, data.len()) {
            on_resize(self);
        }

        self.write(gpu, 0, data)
    }
}
