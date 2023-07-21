use std::{borrow::Cow, str::FromStr, sync::Arc};

use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    texture::Texture,
    texture_loaders::TextureFromUrl,
    typed_buffer::TypedBuffer,
    wgsl_utils::wgsl_interpolate,
};
use ambient_std::{
    asset_cache::{AssetCache, AssetKeepalive, AsyncAssetKey, AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    include_file,
};
use ambient_terrain::{wgsl_terrain_preprocess, TerrainSize, TerrainState};
use async_trait::async_trait;
use bytemuck::Zeroable;
use itertools::Itertools;

use crate::{
    BoxModelKey, NaturalElement, NaturalElementWGSL, NaturalEntity, NATURALS_MAX_ENTITIES,
    OLD_CONTENT_SERVER_URL, WORKGROUP_SIZE,
};

#[derive(Debug, Clone)]
pub struct NaturalsPipelineKey;
#[async_trait]
impl AsyncAssetKey<Arc<NaturalsPipeline>> for NaturalsPipelineKey {
    async fn load(self, assets: AssetCache) -> Arc<NaturalsPipeline> {
        let gpu = GpuKey.get(&assets);
        Arc::new(NaturalsPipeline::new(&gpu, &assets).await)
    }
    fn keepalive(&self) -> AssetKeepalive {
        AssetKeepalive::Forever
    }
}

pub struct NaturalsPipeline {
    pipeline: wgpu::ComputePipeline,
    blue_noise: Arc<Texture>,
    cluster_noise: Arc<Texture>,
    heightmap_sampler: Arc<wgpu::Sampler>,
    default_sampler: Arc<wgpu::Sampler>,
}
impl NaturalsPipeline {
    pub async fn new(gpu: &Gpu, assets: &AssetCache) -> Self {
        let pipeline =
            gpu.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Naturals"),
                    layout: Some(&gpu.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[&gpu.device.create_bind_group_layout(
                                &wgpu::BindGroupLayoutDescriptor {
                                    label: None,
                                    entries: &[
                                        // Heightmap
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 0,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Texture {
                                                sample_type: wgpu::TextureSampleType::Float {
                                                    filterable: true,
                                                },
                                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                                multisampled: false,
                                            },
                                            count: None,
                                        },
                                        // Normalmap
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 1,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Texture {
                                                sample_type: wgpu::TextureSampleType::Float {
                                                    filterable: true,
                                                },
                                                view_dimension: wgpu::TextureViewDimension::D2,
                                                multisampled: false,
                                            },
                                            count: None,
                                        },
                                        // Heightmap sampler
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 2,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Sampler(
                                                wgpu::SamplerBindingType::Filtering,
                                            ),
                                            count: None,
                                        },
                                        // Output entities
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 3,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Storage {
                                                    read_only: false,
                                                },
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                        // Output counts
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 4,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Storage {
                                                    read_only: false,
                                                },
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                        // Blue noise
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 5,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Texture {
                                                sample_type: wgpu::TextureSampleType::Float {
                                                    filterable: false,
                                                },
                                                view_dimension: wgpu::TextureViewDimension::D2,
                                                multisampled: false,
                                            },
                                            count: None,
                                        },
                                        // Cluster noise
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 6,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Texture {
                                                sample_type: wgpu::TextureSampleType::Float {
                                                    filterable: true,
                                                },
                                                view_dimension: wgpu::TextureViewDimension::D2,
                                                multisampled: false,
                                            },
                                            count: None,
                                        },
                                        // Natural elements
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 7,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Buffer {
                                                ty: wgpu::BufferBindingType::Storage {
                                                    read_only: true,
                                                },
                                                has_dynamic_offset: false,
                                                min_binding_size: None,
                                            },
                                            count: None,
                                        },
                                        // Default sampler
                                        wgpu::BindGroupLayoutEntry {
                                            binding: 8,
                                            visibility: wgpu::ShaderStages::COMPUTE,
                                            ty: wgpu::BindingType::Sampler(
                                                wgpu::SamplerBindingType::Filtering,
                                            ),
                                            count: None,
                                        },
                                    ],
                                },
                            )],
                            push_constant_ranges: &[],
                        },
                    )),
                    module: &gpu
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Naturals.shader"),
                            source: wgpu::ShaderSource::Wgsl(Cow::Owned(
                                [
                                    &wgsl_interpolate(),
                                    &wgsl_terrain_preprocess(include_file!("naturals.wgsl"))
                                        .replace(
                                            "NATURALS_MAX_ENTITIES",
                                            &NATURALS_MAX_ENTITIES.to_string(),
                                        )
                                        .replace("WORKGROUP_SIZE", &WORKGROUP_SIZE.to_string()),
                                ]
                                .into_iter()
                                .join("\n"),
                            )),
                        }),
                    entry_point: "main",
                });
        Self {
            pipeline,
            blue_noise: TextureFromUrl {
                url: AbsAssetUrl::from_str(&format!("{OLD_CONTENT_SERVER_URL}assets/models/Misc/FreeBlueNoiseTextures/64_64/HDR_L_0.png"))
                    .unwrap(),
                format: wgpu::TextureFormat::Rgba8Unorm,
            }
            .get(assets)
            .await
            .unwrap(),
            cluster_noise: TextureFromUrl {
                url: AbsAssetUrl::from_str(&format!(
                    "{OLD_CONTENT_SERVER_URL}assets/models/{}",
                    "ArtStationSurfaces/VFX-HQ-Seamless-Noise-Pack-Vol1/Noise_002.png"
                ))
                .unwrap(),
                format: wgpu::TextureFormat::Rgba8Unorm,
            }
            .get(assets)
            .await
            .unwrap(),

            heightmap_sampler: Arc::new(gpu.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })),

            default_sampler: Arc::new(gpu.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            })),
        }
    }
    pub async fn run(
        &self,
        gpu: &Gpu,
        elements: &[(NaturalElement, BoxModelKey)],
        grid_size: f32,
        terrain_state: &TerrainState,
    ) -> Vec<NaturalEntity> {
        let out_count_staging = TypedBuffer::<u32>::new_init(
            gpu,
            "Naturals.out_count_staging",
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            &[0],
        );
        let out_entities_staging = TypedBuffer::<NaturalEntity>::new(
            gpu,
            "Naturals.out_entities_staging",
            NATURALS_MAX_ENTITIES as usize,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );

        {
            let natural_elements = TypedBuffer::<NaturalElementWGSL>::new_init(
                gpu,
                "Naturals.elements",
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                &elements
                    .iter()
                    .map(|(el, _)| NaturalElementWGSL::from(el.clone()))
                    .collect_vec(),
            );

            let out_count_buffer = TypedBuffer::<u32>::new_init(
                gpu,
                "Naturals.out_count",
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                &[0],
            );
            let out_entities_buffer = TypedBuffer::<NaturalEntity>::new(
                gpu,
                "Naturals.out_entities",
                NATURALS_MAX_ENTITIES as usize,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            );

            let bind_group_layout = self.pipeline.get_bind_group_layout(0);
            let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &terrain_state.heightmap.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &terrain_state.normalmap.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.heightmap_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: out_entities_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: out_count_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            &self.blue_noise.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(
                            &self.cluster_noise.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: natural_elements.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Sampler(&self.default_sampler),
                    },
                ],
            });

            let size = TerrainSize::default();
            let cells = (size.texture_size() as f32 / grid_size) as u32;

            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            {
                let mut cpass =
                    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                cpass.set_pipeline(&self.pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(
                    (cells as f32 / WORKGROUP_SIZE as f32).ceil() as u32,
                    cells,
                    1,
                );
            }
            encoder.copy_buffer_to_buffer(
                out_count_buffer.buffer(),
                0,
                out_count_staging.buffer(),
                0,
                4,
            );
            encoder.copy_buffer_to_buffer(
                out_entities_buffer.buffer(),
                0,
                out_entities_staging.buffer(),
                0,
                NATURALS_MAX_ENTITIES as u64 * 4 * 4,
            );

            gpu.queue.submit(Some(encoder.finish()));
        }

        let count =
            out_count_staging.read(gpu, ..).await.unwrap().to_vec()[0].min(NATURALS_MAX_ENTITIES);

        if count > 0 {
            out_entities_staging
                .read(gpu, 0..count as usize)
                .await
                .unwrap()
                .to_vec()
        } else {
            Vec::new()
        }
    }
}
