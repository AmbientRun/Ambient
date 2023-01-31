use std::{
    io::Cursor, num::NonZeroU32, ops::Deref, path::Path, sync::{
        atomic::{AtomicU32, AtomicUsize, Ordering}, Arc
    }
};

use bytemuck::Pod;
use byteorder::{ByteOrder, LittleEndian};
use elements_std::asset_cache::{AssetCache, SyncAssetKeyExt};
use glam::{uvec2, UVec4, Vec4};
use image::{io::Reader as ImageReader, DynamicImage, Rgba, RgbaImage};
use itertools::Itertools;
use ndarray::{s, Array, Array2, Array4, Dimension};
use ordered_float::OrderedFloat;
use wgpu::util::DeviceExt;

use super::{
    fill::FillerKey, gpu::{Gpu, GpuKey}, mipmap::generate_mipmaps
};

static TEXTURE_ALIVE_COUNT: AtomicU32 = AtomicU32::new(0);
static TEXTURE_ID_COUNT: AtomicU32 = AtomicU32::new(0);
static TEXTURES_TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Texture {
    pub gpu: Arc<Gpu>,
    pub id: u32,
    pub label: Option<String>,
    pub handle: wgpu::Texture,
    pub size: wgpu::Extent3d,
    pub size_in_bytes: usize,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub mip_level_count: u32,
}
impl Texture {
    pub fn n_alive() -> u32 {
        TEXTURE_ALIVE_COUNT.load(Ordering::SeqCst)
    }
    pub fn total_bytes_used() -> usize {
        TEXTURES_TOTAL_SIZE.load(Ordering::SeqCst)
    }
    fn size_in_bytes_from_desc(descriptor: &wgpu::TextureDescriptor) -> usize {
        let mut mip_size = (descriptor.size.width * descriptor.size.height * descriptor.size.depth_or_array_layers) as usize
            * descriptor.format.describe().block_size as usize;
        let mut size_in_bytes = mip_size;
        for _ in 1..descriptor.mip_level_count {
            mip_size /= 2;
            size_in_bytes += mip_size;
        }
        size_in_bytes
    }
    pub fn new(gpu: Arc<Gpu>, descriptor: &wgpu::TextureDescriptor) -> Self {
        TEXTURE_ALIVE_COUNT.fetch_add(1, Ordering::SeqCst);
        let id = TEXTURE_ID_COUNT.fetch_add(1, Ordering::SeqCst);
        let size_in_bytes = Self::size_in_bytes_from_desc(descriptor);
        TEXTURES_TOTAL_SIZE.fetch_add(size_in_bytes, Ordering::SeqCst);
        Self {
            id,
            label: descriptor.label.map(|x| x.to_string()),
            size: descriptor.size,
            size_in_bytes,
            format: descriptor.format,
            sample_count: descriptor.sample_count,
            mip_level_count: descriptor.mip_level_count,
            handle: gpu.device.create_texture(descriptor),
            gpu,
        }
    }
    pub fn new_with_data(gpu: Arc<Gpu>, descriptor: &wgpu::TextureDescriptor, data: &[u8]) -> Self {
        TEXTURE_ALIVE_COUNT.fetch_add(1, Ordering::SeqCst);
        let id = TEXTURE_ID_COUNT.fetch_add(1, Ordering::SeqCst);
        let size_in_bytes = Self::size_in_bytes_from_desc(descriptor);
        TEXTURES_TOTAL_SIZE.fetch_add(size_in_bytes, Ordering::SeqCst);
        Self {
            id,
            label: descriptor.label.map(|x| x.to_string()),
            size: descriptor.size,
            size_in_bytes,
            format: descriptor.format,
            sample_count: descriptor.sample_count,
            mip_level_count: descriptor.mip_level_count,
            handle: gpu.device.create_texture_with_data(&gpu.queue, descriptor, data),
            gpu,
        }
    }
    pub fn from_file<P: AsRef<Path> + std::fmt::Debug>(gpu: Arc<Gpu>, path: P, format: wgpu::TextureFormat) -> Self {
        let label = format!("{path:?}");
        Self::from_image(gpu, ImageReader::open(path).unwrap().decode().unwrap(), format, Some(&label))
    }
    pub fn from_image_mipmapped(assets: AssetCache, image: DynamicImage, format: wgpu::TextureFormat, label: wgpu::Label) -> Self {
        Self::from_rgba8_image_mipmapped(assets, &image.to_rgba8(), format, label)
    }
    pub fn from_rgba8_image_mipmapped(
        assets: AssetCache,
        image: &image::RgbaImage,
        format: wgpu::TextureFormat,
        label: wgpu::Label,
    ) -> Self {
        let gpu = GpuKey.get(&assets);

        let size_max = image.width().max(image.height());
        let mip_levels = size_max.ilog2().max(1);

        let texture = Self::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d { width: image.width(), height: image.height(), depth_or_array_layers: 1 },
                mip_level_count: mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label,
            },
        );
        texture.write(image.as_raw());
        let mut encoder =
            gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Texture.from_image_mipmapped") });
        generate_mipmaps(assets, &mut encoder, &texture.handle, texture.format, mip_levels, 0);
        gpu.queue.submit(Some(encoder.finish()));
        texture
    }
    pub fn from_image(gpu: Arc<Gpu>, image: DynamicImage, format: wgpu::TextureFormat, label: wgpu::Label) -> Self {
        let img = image.into_rgba8();

        Self::new_with_data(
            gpu,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d { width: img.width(), height: img.height(), depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label,
            },
            &img.into_vec(),
        )
    }
    /// This will automatically resize the images to the largest size if they're not the same size
    pub fn array_rgba8_mipmapped(assets: AssetCache, label: Option<&str>, mut data: Vec<RgbaImage>, format: wgpu::TextureFormat) -> Self {
        let gpu = GpuKey.get(&assets);
        let layers = data.len();

        let min_size = data.iter().map(|x| uvec2(x.width(), x.height())).reduce(|p, x| p.min(x)).unwrap_or_default();
        let max_size = data.iter().map(|x| uvec2(x.width(), x.height())).reduce(|p, x| p.max(x)).unwrap_or_default();
        if min_size != max_size {
            for img in &mut data {
                image::imageops::resize(img, max_size.x, max_size.y, image::imageops::FilterType::CatmullRom);
            }
        }

        let size_max = data[0].width().max(data[0].height());
        let mip_levels = size_max.ilog2();

        let texture = Self::new(
            gpu.clone(),
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d { width: data[0].width(), height: data[0].height(), depth_or_array_layers: layers as u32 },
                mip_level_count: mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label,
            },
        );
        for (layer, img) in data.into_iter().enumerate() {
            texture.gpu.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture.handle,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                &img.into_vec(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(texture.size.width * texture.format.describe().block_size as u32),
                    rows_per_image: NonZeroU32::new(texture.size.height),
                },
                wgpu::Extent3d { width: texture.size.width, height: texture.size.height, depth_or_array_layers: 1 },
            );
        }

        let mut encoder =
            gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Texture.array_rgba8_mipmapped") });
        for layer in 0..layers {
            generate_mipmaps(assets.clone(), &mut encoder, &texture.handle, texture.format, mip_levels, layer as u32);
        }
        gpu.queue.submit(Some(encoder.finish()));
        texture
    }

    pub fn array_from_files<P: AsRef<Path> + std::fmt::Debug>(assets: AssetCache, paths: Vec<P>, format: wgpu::TextureFormat) -> Self {
        let imgs = paths.iter().map(|path| ImageReader::open(path).unwrap().decode().unwrap().into_rgba8()).collect_vec();

        let name = paths.iter().map(|x| format!("{x:?}")).join(", ");
        Self::array_rgba8_mipmapped(assets, Some(&name), imgs, format)
    }

    pub fn from_array2(gpu: Arc<Gpu>, data: &Array2<f32>) -> Self {
        Self::new_with_data(
            gpu,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d { width: data.shape()[0] as u32, height: data.shape()[1] as u32, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("texture"),
            },
            bytemuck::cast_slice(data.as_slice().unwrap()),
        )
    }

    pub fn write_array2(&self, data: &Array2<f32>) {
        let size = wgpu::Extent3d { width: data.shape()[0] as u32, height: data.shape()[1] as u32, depth_or_array_layers: 1 };
        self.gpu.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &self.handle, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            bytemuck::cast_slice(data.as_slice().unwrap()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * size.width),
                rows_per_image: NonZeroU32::new(size.height),
            },
            size,
        );
    }

    pub fn write_array<A: Pod, D: Dimension>(&self, data: &Array<A, D>) {
        self.write(bytemuck::cast_slice(data.as_slice().unwrap()));
    }
    pub fn write(&self, data: &[u8]) {
        self.gpu.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &self.handle, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(self.size.width * self.format.describe().block_size as u32),
                rows_per_image: NonZeroU32::new(self.size.height),
            },
            self.size,
        );
    }

    pub fn reader(&self) -> TextureReader {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let reader = self.reader_with_encoder(&mut encoder);
        self.gpu.queue.submit(Some(encoder.finish()));
        reader
    }
    pub fn reader_with_encoder(&self, encoder: &mut wgpu::CommandEncoder) -> TextureReader {
        let reader = TextureReader::new(self.gpu.clone(), self.size, self.sample_count, self.format);
        reader.copy_texture_with_encoder(&self.handle, encoder);
        reader
    }

    pub fn new_single_color_texture(gpu: Arc<Gpu>, color: UVec4) -> Self {
        Self::new_with_data(
            gpu,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("Texture.new_single_color_texture"),
            },
            bytemuck::cast_slice(&[color.x as u8, color.y as u8, color.z as u8, color.w as u8]),
        )
    }

    pub fn new_single_color_texture_array(gpu: Arc<Gpu>, colors: Vec<UVec4>) -> Self {
        Self::new_with_data(
            gpu,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: colors.len() as u32 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("default_texture"),
            },
            bytemuck::cast_slice(
                &colors.into_iter().flat_map(|color| vec![color.x as u8, color.y as u8, color.z as u8, color.w as u8]).collect_vec(),
            ),
        )
    }
    pub fn generate_mipmaps(&self, assets: AssetCache) {
        let mut encoder =
            self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Texture.generate_mipmaps") });
        self.generate_mipmaps_with_encoder(assets, &mut encoder);
        self.gpu.queue.submit(Some(encoder.finish()));
    }

    pub fn generate_mipmaps_with_encoder(&self, assets: AssetCache, encoder: &mut wgpu::CommandEncoder) {
        for l in 0..self.size.depth_or_array_layers {
            generate_mipmaps(assets.clone(), encoder, &self.handle, self.format, self.mip_level_count, l);
        }
    }
    pub fn fill(&self, assets: AssetCache, color: Vec4) {
        FillerKey { format: self.format }.get(&assets).run(&self.handle.create_view(&Default::default()), self.size, color);
    }
    pub fn create_view(self: &Arc<Self>, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView { handle: self.handle.create_view(desc), texture: self.clone() }
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        TEXTURE_ALIVE_COUNT.fetch_sub(1, Ordering::SeqCst);
        TEXTURES_TOTAL_SIZE.fetch_sub(self.size_in_bytes, Ordering::SeqCst);
    }
}

/// Wraps wgpu::TextureView, but also keeps a reference to the Texture, both so that we can
/// access information from the Texture (size etc.), but also so that the Texture is kept alive
/// for the asset cache
#[derive(Debug)]
pub struct TextureView {
    pub handle: wgpu::TextureView,
    pub texture: Arc<Texture>,
}
impl Deref for TextureView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

pub struct TextureReader {
    gpu: Arc<Gpu>,
    staging_output_buffer: wgpu::Buffer,
    buffer_dimensions: WgpuBufferDimensions,
    base_size: wgpu::Extent3d,
    size: wgpu::Extent3d,
    _sample_count: u32,
    format: wgpu::TextureFormat,
}
impl TextureReader {
    pub fn new(gpu: Arc<Gpu>, base_size: wgpu::Extent3d, sample_count: u32, format: wgpu::TextureFormat) -> Self {
        let block_size = format.describe().block_size as usize;
        let size = wgpu::Extent3d {
            width: base_size.width * sample_count,
            height: base_size.height * sample_count,
            depth_or_array_layers: base_size.depth_or_array_layers,
        };
        let buffer_dimensions = WgpuBufferDimensions::new(size, block_size);
        Self {
            staging_output_buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (buffer_dimensions.padded_size) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            buffer_dimensions,
            base_size,
            size,
            _sample_count: sample_count,
            format,
            gpu,
        }
    }
    pub fn copy_texture(&self, texture: &wgpu::Texture) {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.copy_texture_with_encoder(texture, &mut encoder);
        self.gpu.queue.submit(Some(encoder.finish()));
    }
    pub fn copy_texture_with_encoder(&self, texture: &wgpu::Texture, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::ImageCopyBuffer {
                buffer: &self.staging_output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::num::NonZeroU32::new(self.buffer_dimensions.padded_bytes_per_row as u32).unwrap()),
                    rows_per_image: Some(std::num::NonZeroU32::new(self.buffer_dimensions.size.height).unwrap()),
                },
            },
            self.base_size,
        );
    }

    /// Reads the whole texture async
    pub async fn read(&self) -> Option<Vec<u8>> {
        let buffer_slice = self.staging_output_buffer.slice(..);
        let (tx, buffer_future) = tokio::sync::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, |v| {
            tx.send(v).ok();
        });

        if !self.gpu.will_be_polled {
            self.gpu.device.poll(wgpu::Maintain::Wait);
        }

        if let Ok(()) = buffer_future.await.unwrap() {
            let padded_buffer = buffer_slice.get_mapped_range();
            let mut result = vec![
                0;
                self.size.width as usize
                    * self.size.height as usize
                    * self.size.depth_or_array_layers as usize
                    * self.format.describe().block_size as usize
            ];
            for (i, chunk) in padded_buffer.chunks(self.buffer_dimensions.padded_bytes_per_row).enumerate() {
                result[(i * self.buffer_dimensions.unpadded_bytes_per_row)..((i + 1) * self.buffer_dimensions.unpadded_bytes_per_row)]
                    .copy_from_slice(&chunk[0..self.buffer_dimensions.unpadded_bytes_per_row]);
            }
            drop(padded_buffer);
            self.staging_output_buffer.unmap();
            Some(result)
        } else {
            None
        }
    }

    pub async fn read_array_f32(&self) -> Option<Array4<f32>> {
        if let Some(bytes) = self.read().await {
            let mut numbers = vec![
                0.;
                self.size.width as usize
                    * self.size.height as usize
                    * self.size.depth_or_array_layers as usize
                    * self.format.n_channels()
            ];
            LittleEndian::read_f32_into(&bytes, &mut numbers);
            Some(
                Array4::from_shape_vec(
                    (
                        self.size.depth_or_array_layers as usize,
                        self.size.width as usize,
                        self.size.height as usize,
                        self.format.n_channels(),
                    ),
                    numbers,
                )
                .unwrap(),
            )
        } else {
            None
        }
    }
    pub async fn read_image(&self) -> Option<DynamicImage> {
        self.read_images().await.map(|mut images| images.pop().unwrap())
    }
    pub async fn read_png(&self) -> Option<Vec<u8>> {
        self.read_image().await.and_then(|image| {
            let mut data = Cursor::new(Vec::new());
            image.write_to(&mut data, image::ImageOutputFormat::Png).ok()?;
            Some(data.into_inner())
        })
    }
    pub async fn read_images(&self) -> Option<Vec<DynamicImage>> {
        if self.format == wgpu::TextureFormat::R32Float {
            let array = self.read_array_f32().await?;
            Some(
                (0..self.size.depth_or_array_layers as usize)
                    .map(|layer| {
                        // println!("reading {layer}");
                        let data = array.slice(s![layer, .., .., ..]);
                        let max = *data.iter().map(|x| OrderedFloat(*x)).max().unwrap();
                        let min = *data.iter().map(|x| OrderedFloat(*x)).min().unwrap();
                        let as_u8s = data.iter().map(|v| (255. * (v - min) / (max - min)) as u8).collect_vec();
                        // println!("min={min} max={max}");
                        match self.format {
                            wgpu::TextureFormat::Depth32Float => {
                                DynamicImage::ImageLuma8(image::GrayImage::from_raw(self.size.width, self.size.height, as_u8s).unwrap())
                            }
                            _ => todo!(),
                        }
                    })
                    .collect_vec(),
            )
        } else if self.format == wgpu::TextureFormat::Rgba8UnormSrgb {
            let data = self.read().await?;
            Some(
                data.chunks((self.size.width * self.size.height * 4) as usize)
                    .map(|chunk_data| {
                        let img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                            self.size.width,
                            self.size.height,
                            chunk_data.to_vec(),
                        )
                        .unwrap();
                        DynamicImage::ImageRgba8(img)
                    })
                    .collect_vec(),
            )
        } else if self.format == wgpu::TextureFormat::Bgra8UnormSrgb {
            let data = self.read().await?;
            Some(
                data.chunks((self.size.width * self.size.height * 4) as usize)
                    .map(|chunk_data| {
                        let mut img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
                            self.size.width,
                            self.size.height,
                            chunk_data.to_vec(),
                        )
                        .unwrap();
                        for pixel in img.pixels_mut() {
                            let Rgba([b, g, r, a]) = *pixel;
                            *pixel = Rgba([r, b, g, a]);
                        }
                        DynamicImage::ImageRgba8(img)
                    })
                    .collect_vec(),
            )
        } else {
            unimplemented!("{:?}", self.format)
        }
    }
    pub async fn write_to_file(&self, path: impl AsRef<Path>) {
        let image = self.read_image().await.unwrap().into_rgba8();
        image.save(path).unwrap();
    }
    pub async fn write_to_files(&self, path: &str) {
        let images = self.read_images().await.unwrap();
        for (i, image) in images.into_iter().enumerate() {
            image.save(&format!("{path}_{i}.png")).unwrap();
        }
    }
}

// From: https://github.com/gfx-rs/wgpu/blob/master/wgpu/examples/capture/main.rs#L174
pub struct WgpuBufferDimensions {
    pub size: wgpu::Extent3d,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
    pub padded_size: usize,
}

impl WgpuBufferDimensions {
    pub fn new(size: wgpu::Extent3d, bytes_per_pixel: usize) -> Self {
        let unpadded_bytes_per_row = (size.width as usize) * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            size,
            padded_size: padded_bytes_per_row * size.height as usize * size.depth_or_array_layers as usize,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }
}

trait NTextureChannels {
    fn n_channels(&self) -> usize;
}
impl NTextureChannels for wgpu::TextureFormat {
    fn n_channels(&self) -> usize {
        match self {
            wgpu::TextureFormat::R8Unorm => 1,
            wgpu::TextureFormat::R8Snorm => 1,
            wgpu::TextureFormat::R8Uint => 1,
            wgpu::TextureFormat::R8Sint => 1,
            wgpu::TextureFormat::R16Uint => 1,
            wgpu::TextureFormat::R16Sint => 1,
            wgpu::TextureFormat::R16Float => 1,
            wgpu::TextureFormat::Rg8Unorm => 2,
            wgpu::TextureFormat::Rg8Snorm => 2,
            wgpu::TextureFormat::Rg8Uint => 2,
            wgpu::TextureFormat::Rg8Sint => 2,
            wgpu::TextureFormat::R32Uint => 1,
            wgpu::TextureFormat::R32Sint => 1,
            wgpu::TextureFormat::R32Float => 1,
            wgpu::TextureFormat::Rg16Uint => 2,
            wgpu::TextureFormat::Rg16Sint => 2,
            wgpu::TextureFormat::Rg16Float => 2,
            wgpu::TextureFormat::Rgba8Unorm => 4,
            wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::Rgba8Snorm => 4,
            wgpu::TextureFormat::Rgba8Uint => 4,
            wgpu::TextureFormat::Rgba8Sint => 4,
            wgpu::TextureFormat::Bgra8Unorm => 4,
            wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            wgpu::TextureFormat::Rgb10a2Unorm => 4,
            wgpu::TextureFormat::Rg32Uint => 2,
            wgpu::TextureFormat::Rg32Sint => 2,
            wgpu::TextureFormat::Rg32Float => 2,
            wgpu::TextureFormat::Rgba16Uint => 4,
            wgpu::TextureFormat::Rgba16Sint => 4,
            wgpu::TextureFormat::Rgba16Float => 4,
            wgpu::TextureFormat::Rgba32Uint => 4,
            wgpu::TextureFormat::Rgba32Sint => 4,
            wgpu::TextureFormat::Rgba32Float => 4,
            wgpu::TextureFormat::Depth32Float => 1,
            _ => todo!(),
        }
    }
}
