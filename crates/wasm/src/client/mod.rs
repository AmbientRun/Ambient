use crate::shared::{self, client_bytecode_from_url, module_bytecode, ModuleBytecode};
use ambient_core::{asset_cache, async_ecs::async_run, runtime};
use ambient_ecs::{query, EntityId, SystemGroup, World};
use ambient_guest_bridge::components::text::hyperlink;
use ambient_std::{
    asset_cache::AsyncAssetKeyExt, asset_url::AbsAssetUrl, download_asset::BytesFromUrl,
};
use std::{str::FromStr, sync::Arc};

mod implementation;
mod network;

pub fn initialize(
    world: &mut World,
    messenger: Arc<dyn Fn(&World, EntityId, shared::MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    shared::initialize(world, messenger, |id| Bindings {
        base: Default::default(),
        world_ref: Default::default(),
        id,
    })?;

    network::initialize(world);

    Ok(())
}
pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "core/wasm/client",
        vec![
            query(hyperlink().changed()).to_system(move |q, world, qs, _| {
                for (id, url) in q.collect_cloned(world, qs) {
                    webbrowser::open(&url).ok();
                    world.despawn(id);
                }
            }),
            query(client_bytecode_from_url().changed()).to_system(move |q, world, qs, _| {
                for (id, url) in q.collect_cloned(world, qs) {
                    let url = match AbsAssetUrl::from_str(&url) {
                        Ok(value) => value,
                        Err(err) => {
                            log::warn!("Failed to parse client_bytecode_from_url url: {:?}", err);
                            continue;
                        }
                    };
                    let assets = world.resource(asset_cache()).clone();
                    let async_run = world.resource(async_run()).clone();
                    world.resource(runtime()).spawn(async move {
                        match BytesFromUrl::new(url, true).get(&assets).await {
                            Err(err) => {
                                log::warn!("Failed to load client bytecode from url: {:?}", err);
                            }
                            Ok(bytecode) => {
                                async_run.run(move |world| {
                                    world
                                        .add_component(
                                            id,
                                            module_bytecode(),
                                            ModuleBytecode(bytecode.to_vec()),
                                        )
                                        .ok();
                                });
                            }
                        }
                    });
                }
            }),
            Box::new(shared::systems()),
        ],
    )
}

#[derive(Clone)]
struct Bindings {
    base: shared::bindings::BindingsBase,
    world_ref: shared::bindings::WorldRef,
    id: EntityId,
}
impl Bindings {
    pub fn world(&self) -> &World {
        unsafe { self.world_ref.world() }
    }
    pub fn world_mut(&mut self) -> &mut World {
        unsafe { self.world_ref.world_mut() }
    }
}

impl shared::bindings::BindingsBound for Bindings {
    fn base(&self) -> &shared::bindings::BindingsBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut shared::bindings::BindingsBase {
        &mut self.base
    }
    fn set_world(&mut self, world: &mut World) {
        unsafe {
            self.world_ref.set_world(world);
        }
    }
    fn clear_world(&mut self) {
        unsafe {
            self.world_ref.clear_world();
        }
    }
}
