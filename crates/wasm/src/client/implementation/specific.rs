//! Used to implement all the client-specific host functions.
//!
//! If implementing a trait that is also available on the server, it should go in [super].

use ambient_audio::AudioFromUrl;
use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    player::local_user_id,
    runtime,
    window::{window_ctl, WindowCtl},
};
use ambient_input::{player_prev_raw_input, player_raw_input};
use ambient_network::client::game_client;
use ambient_std::{asset_cache::AsyncAssetKeyExt, asset_url::AbsAssetUrl};
use ambient_world_audio::{audio_sender, AudioMessage};
use anyhow::Context;
use winit::window::CursorGrabMode;

use super::Bindings;
use crate::shared::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::message,
    wit,
};

use ambient_core::camera::clip_space_ray;

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
}
impl wit::client_audio::Host for Bindings {
    fn load(&mut self, url: String) -> anyhow::Result<()> {
        let world = self.world();
        let assets = world.resource(asset_cache()).clone();
        let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
        let audio_url = AudioFromUrl {
            url: AbsAssetUrl::parse(asset_url).context("Failed to parse audio url")?,
        };
        let _track = audio_url.peek(&assets);
        Ok(())
    }

    fn play(&mut self, url: String, looping: bool, volume: f32, uid: u32) -> anyhow::Result<()> {
        let world = self.world();
        let assets = world.resource(asset_cache()).clone();
        let asset_url = AbsAssetUrl::from_asset_key(url).to_string();
        let audio_url = AudioFromUrl {
            url: AbsAssetUrl::parse(asset_url.clone()).context("Failed to parse audio url")?,
        };
        let runtime = world.resource(runtime()).clone();
        let async_run = world.resource(async_run()).clone();
        runtime.spawn(async move {
            let track = audio_url.get(&assets).await;
            async_run.run(move |world| {
                match track {
                    Ok(track) => {
                        let sender = world.resource(audio_sender());
                        sender
                            .send(AudioMessage::Track(
                                track,
                                looping,
                                volume,
                                asset_url.replace("ambient-assets:/", ""),
                                uid,
                            ))
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
