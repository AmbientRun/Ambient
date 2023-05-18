//! Used to implement all the client-specific host functions.
//!
//! If implementing a trait that is also available on the server, it should go in [super].

use std::sync::Arc;

use ambient_audio::AudioFromUrl;
use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    player::local_user_id,
    runtime,
    window::{window_ctl, WindowCtl},
};
use ambient_gpu::{gpu::GpuKey, texture::Texture};
use ambient_input::{player_prev_raw_input, player_raw_input};
use ambient_network::client::game_client;
use ambient_procedurals::procedural_storage;
use ambient_renderer::pbr_material::{PbrMaterialConfig, PbrMaterialParams};
use ambient_std::{
    asset_cache::{AsyncAssetKeyExt, SyncAssetKeyExt},
    asset_url::AbsAssetUrl,
    mesh::MeshBuilder,
};
use ambient_world_audio::{audio_sender, AudioMessage};
use anyhow::Context;
use glam::Vec4;
use wgpu::TextureViewDescriptor;
use winit::window::CursorGrabMode;

use super::Bindings;
use crate::shared::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::message,
    wit,
};

use ambient_core::camera::{clip_space_ray, world_to_clip_space};

impl wit::client_message::Host for Bindings {
    fn send(
        &mut self,
        target: wit::client_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        use wit::client_message::Target;
        let module_id = self.id;
        let world = self.world_mut();

        match target {
            Target::ServerUnreliable | Target::ServerReliable => {
                let connection = world
                    .resource(game_client())
                    .as_ref()
                    .context("no game client")?
                    .connection
                    .clone();

                message::send_networked(
                    world,
                    connection,
                    module_id,
                    &name,
                    &data,
                    matches!(target, Target::ServerReliable),
                )
            }
            Target::LocalBroadcast => message::send_local(world, module_id, None, name, data),
            Target::Local(id) => {
                message::send_local(world, module_id, Some(id.from_bindgen()), name, data)
            }
        }
    }
}
impl wit::client_player::Host for Bindings {
    fn get_local(&mut self) -> anyhow::Result<wit::types::EntityId> {
        crate::shared::implementation::player::get_by_user_id(
            self.world(),
            self.world().resource(local_user_id()).clone(),
        )
        .transpose()
        .unwrap()
    }
}
impl wit::client_input::Host for Bindings {
    fn get(&mut self) -> anyhow::Result<wit::client_input::Input> {
        Ok(self
            .world()
            .resource(player_raw_input())
            .clone()
            .into_bindgen())
    }

    fn get_previous(&mut self) -> anyhow::Result<wit::client_input::Input> {
        Ok(self
            .world()
            .resource(player_prev_raw_input())
            .clone()
            .into_bindgen())
    }

    fn set_cursor(&mut self, icon: wit::client_input::CursorIcon) -> anyhow::Result<()> {
        Ok(self
            .world()
            .resource(ambient_core::window::window_ctl())
            .send(ambient_core::window::WindowCtl::SetCursorIcon(
                icon.from_bindgen().into(),
            ))?)
    }

    fn set_cursor_visible(&mut self, visible: bool) -> anyhow::Result<()> {
        Ok(self
            .world()
            .resource(ambient_core::window::window_ctl())
            .send(ambient_core::window::WindowCtl::ShowCursor(visible))?)
    }

    fn set_cursor_lock(&mut self, lock: bool) -> anyhow::Result<()> {
        let grab_mode = if lock {
            if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
                CursorGrabMode::Confined
            } else if cfg!(target_os = "macos") {
                CursorGrabMode::Locked
            } else {
                anyhow::bail!("Unsupported platform for cursor lock.")
            }
        } else {
            CursorGrabMode::None
        };

        Ok(self
            .world()
            .resource(ambient_core::window::window_ctl())
            .send(ambient_core::window::WindowCtl::GrabCursor(grab_mode))?)
    }
}
impl wit::client_camera::Host for Bindings {
    fn clip_space_ray(
        &mut self,
        camera: wit::types::EntityId,
        clip_space_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Ray> {
        let mut ray = clip_space_ray(
            self.world(),
            camera.from_bindgen(),
            clip_space_pos.from_bindgen(),
        )?;
        ray.dir *= -1.;
        Ok(ray.into_bindgen())
    }

    fn screen_to_clip_space(
        &mut self,
        screen_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Vec2> {
        Ok(
            ambient_core::window::screen_to_clip_space(self.world(), screen_pos.from_bindgen())
                .into_bindgen(),
        )
    }

    fn screen_to_world_direction(
        &mut self,
        camera: wit::types::EntityId,
        screen_pos: wit::types::Vec2,
    ) -> anyhow::Result<wit::types::Ray> {
        let clip_space =
            ambient_core::window::screen_to_clip_space(self.world(), screen_pos.from_bindgen());
        let mut ray = clip_space_ray(self.world(), camera.from_bindgen(), clip_space)?;
        ray.dir *= -1.;
        Ok(ray.into_bindgen())
    }

    fn world_to_screen(
        &mut self,
        camera: wit::types::EntityId,
        world_pos: wit::types::Vec3,
    ) -> anyhow::Result<wit::types::Vec2> {
        let clip_pos = world_to_clip_space(
            self.world(),
            camera.from_bindgen(),
            world_pos.from_bindgen(),
        )?;
        Ok(ambient_core::window::clip_to_screen_space(self.world(), clip_pos).into_bindgen())
    }
}
impl wit::client_audio::Host for Bindings {
    fn load(&mut self, url: String) -> anyhow::Result<()> {
        let world = self.world();
        let assets = world.resource(asset_cache());
        let audio_url = AudioFromUrl {
            url: AbsAssetUrl::parse(url)?,
        };
        let _track = audio_url.peek(assets);
        Ok(())
    }

    fn play(&mut self, url: String, looping: bool, volume: f32, uid: u32) -> anyhow::Result<()> {
        let world = self.world();
        let assets = world.resource(asset_cache()).clone();
        let runtime = world.resource(runtime()).clone();
        let async_run = world.resource(async_run()).clone();
        let url = AbsAssetUrl::parse(url)?.to_download_url(&assets)?;
        runtime.spawn(async move {
            let track = AudioFromUrl { url: url.clone() }.get(&assets).await;
            async_run.run(move |world| {
                match track {
                    Ok(track) => {
                        let sender = world.resource(audio_sender());
                        sender
                            .send(AudioMessage::Track(track, looping, volume, url, uid))
                            .unwrap();
                    }
                    Err(e) => log::error!("{e:?}"),
                };
            });
        });
        Ok(())
    }

    fn stop(&mut self, url: String) -> anyhow::Result<()> {
        let world = self.world();
        let runtime = world.resource(runtime()).clone();
        let async_run = world.resource(async_run()).clone();
        let assets = world.resource(asset_cache());
        let url = AbsAssetUrl::parse(url)?.to_download_url(assets)?;
        runtime.spawn(async move {
            async_run.run(move |world| {
                let sender = world.resource(audio_sender());
                sender.send(AudioMessage::Stop(url)).unwrap();
            });
        });
        Ok(())
    }

    fn set_volume(&mut self, url: String, volume: f32) -> anyhow::Result<()> {
        let world = self.world();
        let runtime = world.resource(runtime()).clone();
        let async_run = world.resource(async_run()).clone();
        let assets = world.resource(asset_cache());
        let url = AbsAssetUrl::parse(url)?.to_download_url(assets)?;
        runtime.spawn(async move {
            async_run.run(move |world| {
                let sender = world.resource(audio_sender());
                sender
                    .send(AudioMessage::UpdateVolume(url, volume))
                    .unwrap();
            });
        });
        Ok(())
    }

    fn stop_by_id(&mut self, uid: u32) -> anyhow::Result<()> {
        let world = self.world();
        let runtime = world.resource(runtime()).clone();
        let async_run = world.resource(async_run()).clone();
        runtime.spawn(async move {
            async_run.run(move |world| {
                let sender = world.resource(audio_sender());
                sender.send(AudioMessage::StopById(uid)).unwrap();
            });
        });
        Ok(())
    }
}
impl wit::client_window::Host for Bindings {
    fn set_fullscreen(&mut self, fullscreen: bool) -> anyhow::Result<()> {
        self.world_mut()
            .resource(window_ctl())
            .send(WindowCtl::SetFullscreen(fullscreen))?;
        Ok(())
    }
}
impl wit::client_mesh::Host for Bindings {
    fn create(
        &mut self,
        vertices: Vec<wit::client_mesh::Vertex>,
        indices: Vec<u32>,
    ) -> anyhow::Result<wit::client_mesh::Handle> {
        let mut positions = Vec::with_capacity(vertices.len());
        let mut normals = Vec::with_capacity(vertices.len());
        let mut tangents = Vec::with_capacity(vertices.len());
        let mut texcoords = Vec::with_capacity(vertices.len());
        for v in &vertices {
            positions.push(v.position.from_bindgen());
            normals.push(v.normal.from_bindgen());
            tangents.push(v.tangent.from_bindgen());
            texcoords.push(v.texcoord0.from_bindgen());
        }
        let mesh = MeshBuilder {
            positions,
            normals,
            tangents,
            texcoords: vec![texcoords],
            indices,
            ..MeshBuilder::default()
        }
        .build()?;

        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        let mesh_handle = storage.insert_mesh(mesh);
        Ok(mesh_handle.into_bindgen())
    }
}
impl wit::client_material::Host for Bindings {
    fn create(
        &mut self,
        desc: wit::client_material::Descriptor,
    ) -> anyhow::Result<wit::client_material::Handle> {
        let world = self.world_mut();
        let storage = world.resource_mut(procedural_storage());
        let material = PbrMaterialConfig {
            source: "Procedural Material".to_string(),
            name: "Procedural Material".to_string(),
            params: PbrMaterialParams {
                base_color_factor: Vec4::ONE,
                emissive_factor: Vec4::ZERO,
                alpha_cutoff: 0.0,
                metallic: 1.0,
                roughness: 1.0,
                ..PbrMaterialParams::default()
            },
            base_color: Arc::clone(storage.get_texture(desc.base_color_map.from_bindgen())),
            normalmap: Arc::clone(storage.get_texture(desc.normal_map.from_bindgen())),
            metallic_roughness: Arc::clone(
                storage.get_texture(desc.metallic_roughness_map.from_bindgen()),
            ),
            sampler: Arc::clone(storage.get_sampler(desc.sampler.from_bindgen())),
            transparent: false,
            double_sided: false,
            depth_write_enabled: true,
        };
        let material_handle = storage.insert_material(material);
        Ok(material_handle.into_bindgen())
    }
}
impl wit::client_sampler::Host for Bindings {
    fn create(
        &mut self,
        desc: wit::client_sampler::Descriptor,
    ) -> wasmtime::Result<wit::client_sampler::Handle> {
        let address_mode_from_wit = |wit: wit::client_sampler::AddressMode| -> wgpu::AddressMode {
            match wit {
                wit::client_sampler::AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                wit::client_sampler::AddressMode::Repeat => wgpu::AddressMode::Repeat,
                wit::client_sampler::AddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
            }
        };
        let filter_mode_from_wit = |wit: wit::client_sampler::FilterMode| match wit {
            wit::client_sampler::FilterMode::Nearest => wgpu::FilterMode::Nearest,
            wit::client_sampler::FilterMode::Linear => wgpu::FilterMode::Linear,
        };

        let world = self.world_mut();
        let assets = world.resource(asset_cache());
        let gpu = GpuKey.get(assets);
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode_from_wit(desc.address_mode_u),
            address_mode_v: address_mode_from_wit(desc.address_mode_v),
            address_mode_w: address_mode_from_wit(desc.address_mode_w),
            mag_filter: filter_mode_from_wit(desc.mag_filter),
            min_filter: filter_mode_from_wit(desc.min_filter),
            mipmap_filter: filter_mode_from_wit(desc.mipmap_filter),
            ..wgpu::SamplerDescriptor::default()
        });
        let sampler = Arc::new(sampler);
        let storage = world.resource_mut(procedural_storage());
        let sampler_handle = storage.insert_sampler(sampler);
        Ok(sampler_handle.into_bindgen())
    }
}
impl wit::client_texture::Host for Bindings {
    fn create2d(
        &mut self,
        width: u32,
        height: u32,
        format: wit::client_texture::Format,
        data: Vec<u8>,
    ) -> anyhow::Result<wit::client_texture::Handle> {
        let world = self.world_mut();
        let assets = world.resource(asset_cache());
        let gpu = GpuKey.get(assets);
        let format = match format {
            wit::client_texture::Format::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        };
        let texture = Texture::new_with_data(
            gpu,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            &data,
        );
        let texture = Arc::new(texture);
        let texture_view = Arc::new(texture.create_view(&TextureViewDescriptor::default()));
        let storage = world.resource_mut(procedural_storage());
        let texture_handle = storage.insert_texture(texture_view);
        Ok(texture_handle.into_bindgen())
    }
}
