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

use ambient_ecs::{World, Entity, EntityId};
use ambient_primitives::{cube, quad};
use ambient_core::transform::{translation, scale};
use ambient_renderer::color;
use glam::{Vec3, Vec4};

use super::Bindings;
use crate::shared::{
    conversion::{FromBindgen, IntoBindgen},
    implementation::message,
    wit,
};

use ambient_core::camera::{clip_space_ray, world_to_clip_space};

use std::{sync::Arc};
use parking_lot::Mutex;
use rhai::{NativeCallContext, FnPtr, Dynamic};

use slotmap::SlotMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref CONTEXT_MAP: Mutex<SlotMap<slotmap::DefaultKey, rhai::NativeCallContextStore>> = Mutex::new(SlotMap::new());
}


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
        let url = AbsAssetUrl::parse(url)?.to_download_url(&assets)?;
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
        let url = AbsAssetUrl::parse(url)?.to_download_url(&assets)?;
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


impl wit::script::Host for Bindings {
    fn watch(&mut self, path: String) -> anyhow::Result<()> {
        let world = Arc::new(Mutex::new(self.world()));

        let world_runtime = world.lock().resource(runtime()).clone();
        let world_async_run = world.lock().resource(async_run()).clone();
        let world_arc_clone = Arc::clone(&world);
        // UNSAFE: we are using `std::mem::transmute` to change the lifetime of `world_arc_clone`.
        // This is safe as long as the `World` instance is valid during the execution of the Rhai script.
        let world_arc_clone: Arc<Mutex<&'static mut World>> = unsafe { std::mem::transmute(world_arc_clone) };
        world_runtime.spawn(async move {
            let mut last_content = String::new();
            let created_entities = Arc::new(Mutex::new(Vec::<EntityId>::new()));
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let content = reqwest::get(path.clone()).await.unwrap().text().await.unwrap();
                if content != last_content {
                    last_content = content.clone();
                    let world_arc_clone = Arc::clone(&world_arc_clone);
                    for entity_id in created_entities.lock().iter() {
                        let mut world = world_arc_clone.lock();
                        world.despawn(*entity_id);
                    }
                    created_entities.lock().clear();
                    let created_entities_clone = Arc::clone(&created_entities);
                    world_async_run.run(move |_world| {
                        let engine = Arc::new(Mutex::new(rhai::Engine::new()));
                        let keybinds = Arc::new(Mutex::new(std::collections::HashMap::<(String, String, String), FnPtr>::new() ));
                        let keybinds_arc = Arc::clone(&keybinds);
                        let world_arc_1 = Arc::clone(&world_arc_clone);
                        let world_arc_2 = Arc::clone(&world_arc_clone);
                        // let world_arc_3 = Arc::clone(&world_arc_clone);

                        let engine_arc1 = Arc::clone(&engine);
                        let engine_arc2 = Arc::clone(&engine);

                        engine.lock().register_fn("bindkey", move |context: NativeCallContext, key: String, entityid: String, component: String, action: rhai::Dynamic | {

                            let world = Arc::clone(&world_arc_2);
                            let world2 = world.clone();
                            let world_runtime = world.lock().resource(runtime()).clone();
                            let action_fn: FnPtr = action.clone().try_cast::<FnPtr>().unwrap();

                            let engine_arc = Arc::clone(&engine_arc1);
                            let keybinds = Arc::clone(&keybinds_arc);
                            keybinds_arc.lock().insert((key.to_string(), entityid.clone(), component.clone()), action_fn);

                            // Store the context data in the CONTEXT_MAP
                            let context_data = context.store_data();
                            let context_key = CONTEXT_MAP.lock().insert(context_data);

                            let (tx, rx) = flume::unbounded::<&str>();

                            world_runtime.spawn(async move {
                                while let Ok(key) = rx.recv_async().await {
                                    println!("key: {}", key);
                                    let id = EntityId::from_base64(&entityid).unwrap();
                                    let pos = world2.lock().get(id, translation()).unwrap();
                                    let pos_array = pos.to_array().iter().map(|x| Dynamic::from(*x)).collect::<Vec<Dynamic>>();
                                    let context_map = CONTEXT_MAP.lock();
                                    let context_data = context_map.get(context_key).unwrap();
                                    let new_pos_rhai = keybinds.lock().get_mut(&(key.to_string(), entityid.clone(), component.clone())).unwrap().call_within_context::<rhai::Array>(&context_data.create_context(&*engine_arc.lock()), (pos_array,)).unwrap();
                                    let new_pos = Vec3::from_slice(&new_pos_rhai.iter().map(|x| x.as_float().unwrap()).collect::<Vec<f32>>());
                                    world2.lock().set(id, translation(), new_pos).unwrap();
                                }
                            });

                            world_runtime.spawn(async move {
                                loop {
                                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                                    if world.lock().resource(player_raw_input()).keys.contains(&ambient_shared_types::VirtualKeyCode::W) {
                                        if &key == "W" {
                                            tx.send_async("W").await.unwrap();
                                        }
                                    }
                                    if world.lock().resource(player_raw_input()).keys.contains(&ambient_shared_types::VirtualKeyCode::S) {
                                        if &key == "S" {
                                            tx.send_async("S").await.unwrap();
                                        }
                                    }
                                }
                            });
                        });

                        engine.lock().register_fn("new_entity", move |info: rhai::Map| -> String {
                            println!("entity: {:?}", info);
                            let mut entity = Entity::new();
                            let world = Arc::clone(&world_arc_1);
                            if let Some(shape) = info.get("shape") {
                                println!("shape: {:?}", shape);
                                let shape_str = shape.clone().into_string().unwrap();
                                match shape_str.as_str() {
                                    "cube" => {
                                        entity = entity.with_default(cube());
                                    },
                                    "quad" => {
                                        entity = entity.with_default(quad());
                                    },
                                    // "ball" => {
                                    //     entity = entity.with_default(ball());
                                    // },
                                    _ => {}
                                }
                            }
                            if let Some(_translation) = info.get("translation") {
                                println!("translation: {:?}", _translation);
                                let pos = _translation.clone().into_typed_array::<f32>().unwrap();
                                entity = entity.with(translation(), Vec3::from_slice(&pos));
                            }
                            if let Some(_scale) = info.get("scale") {
                                println!("scale: {:?}", _scale);
                                let s = _scale.clone().into_typed_array::<f32>().unwrap();
                                entity = entity.with(scale(), Vec3::from_slice(&s));
                            }
                            if let Some(_color) = info.get("color") {
                                println!("color: {:?}", _color);
                                let c = _color.clone().into_typed_array::<f32>().unwrap();
                                entity = entity.with(color(), Vec4::from_slice(&c));
                            }
                            let mut world = world.lock();
                            let id = entity.spawn(&mut *world);
                            created_entities_clone.lock().push(id);
                            return id.to_base64()
                        });

                        match engine_arc2.lock().run(&content) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("error: {:?}", e);
                            }
                        };
                    });
                }
            }
        });
        Ok(())
    }
}