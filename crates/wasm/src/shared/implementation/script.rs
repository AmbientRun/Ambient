
use ambient_ecs::{World, Entity, EntityId};
use ambient_primitives::{cube, quad};
use ambient_core::transform::{translation, scale};
use ambient_renderer::color;
use std::sync::Arc;
use parking_lot::Mutex;
use ambient_core::{
    async_ecs::async_run,
    runtime,
};
use glam::{Vec3, Vec4};
use crate::shared::implementation::message::*;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
struct RhaiEntityInfo {
    shape: String,
    translation: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
}

pub(crate) fn watch(
    world: Arc<Mutex<&World>>,
    url: String,
) -> anyhow::Result<()> {
    println!("server only now");
    Ok(())
}
