use std::sync::Arc;

use ambient_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    CowStr,
};
use itertools::Itertools;
use wgpu::{BindGroup, BindGroupLayoutEntry, BufferUsages, ShaderStages};

use crate::{
    gpu::{GpuKey, WgslType},
    shader_module::{Shader, ShaderIdent, ShaderModule},
    typed_buffer::TypedBuffer,
};

pub struct GpuRun {
    label: CowStr,
    modules: Vec<Arc<ShaderModule>>,
    body: CowStr,
    bind_groups: Vec<(CowStr, BindGroup)>,
}

impl GpuRun {
    pub fn new(label: impl Into<CowStr>, body: impl Into<CowStr>) -> Self {
        Self { body: body.into(), modules: Default::default(), bind_groups: Default::default(), label: label.into() }
    }

    pub fn add_module(mut self, module: Arc<ShaderModule>) -> Self {
        self.modules.push(module);
        self
    }

    pub fn add_bind_group(mut self, name: impl Into<CowStr>, bind_group: wgpu::BindGroup) -> Self {
        self.bind_groups.push((name.into(), bind_group));
        self
    }

    pub fn into_shader<In: WgslType, Out: WgslType>(&self, assets: &AssetCache) -> Arc<Shader> {
        let Self { body, modules, .. } = self;

        let in_size = std::mem::size_of::<In>() as u64;
        let out_size = std::mem::size_of::<Out>() as u64;

        let in_type = In::wgsl_type();
        let out_type = Out::wgsl_type();

        let module = ShaderModule::new("GpuRun", include_str!("gpu_run.wgsl"))
            .with_binding(
                "GPURUN_BIND_GROUP",
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            )
            .with_binding(
                "GPURUN_BIND_GROUP",
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            )
            .with_ident(ShaderIdent::constant("IN_SIZE", in_size))
            .with_ident(ShaderIdent::constant("OUT_SIZE", out_size))
            .with_ident(ShaderIdent::raw("WGSL_IN", in_type))
            .with_ident(ShaderIdent::raw("WGSL_OUT", out_type))
            .with_ident(ShaderIdent::raw("WGSL_BODY", body.clone()))
            .with_dependencies(modules.iter().cloned());

        Shader::new(
            assets,
            format!("GpuRun.{}", self.label),
            &["GPURUN_BIND_GROUP"].into_iter().chain(self.bind_groups.iter().map(|v| &*v.0)).collect_vec(),
            &module,
        )
        .unwrap()
    }

    pub async fn run<In: WgslType, Out: WgslType>(self, assets: &AssetCache, input: In) -> Out {
        let shader = self.into_shader::<In, Out>(assets);

        let gpu = GpuKey.get(assets);

        let in_buffer = TypedBuffer::new_init(gpu.clone(), "GpuRun.in", BufferUsages::COPY_DST | BufferUsages::STORAGE, &[input]);
        let out_buffer =
            TypedBuffer::new_init(gpu.clone(), "GpuRun.out", BufferUsages::STORAGE | BufferUsages::COPY_SRC, &[Out::zeroed(); 1]);

        let pipeline = shader.to_compute_pipeline(&GpuKey.get(assets), "main");

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GpuRun"),
            layout: shader.layout(0).unwrap(),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: in_buffer.buffer().as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: out_buffer.buffer().as_entire_binding() },
            ],
        });

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            pass.set_pipeline(pipeline.pipeline());
            pass.set_bind_group(0, &bind_group, &[]);

            for (i, (_, v)) in self.bind_groups.iter().enumerate() {
                pass.set_bind_group(i as _, v, &[]);
            }

            pass.dispatch_workgroups(1, 1, 1);
        }

        gpu.queue.submit(Some(encoder.finish()));

        // Only one

        out_buffer.read(.., true).await.expect("Failed to map buffer")[0]
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use ambient_std::asset_cache::{AssetCache, SyncAssetKeyExt};
    use glam::{Vec2, Vec4, Vec4Swizzles};

    use crate::{gpu::GpuKey, gpu_run::GpuRun};

    #[tokio::test]
    async fn test_gpu_run() {
        use crate::gpu::Gpu;
        let gpu = Arc::new(Gpu::new(None).await);
        let assets = AssetCache::new(tokio::runtime::Handle::current());
        GpuKey.insert(&assets, gpu);
        let input = Vec4::ONE;
        let res: Vec2 = GpuRun::new("TestGpuRun", "return (input * 3.).xy;").run(&assets, input).await;
        assert_eq!(res, (input * 3.).xy());
    }
}
