pub use ambient_ecs as ecs;
use ambient_sys::task::spawn_local;
use std::{future::Future, time::Duration};

pub use ecs::{
    generated::{components::core as components, concepts, messages},
    Message, RuntimeMessage,
};

pub fn run_async(world: &ecs::World, future: impl Future<Output = ()> + Send + 'static) {
    world.resource(ambient_core::runtime()).spawn(future);
}

/// Execute a future to completion on a worker thread.
///
/// This permits spawning thread local futures
pub fn run_async_local<F>(_world: &ecs::World, create: impl 'static + Send + FnOnce() -> F)
where
    F: 'static + Future,
    F::Output: Send + 'static,
{
    spawn_local(create);
}

pub async fn sleep(seconds: f32) {
    ambient_sys::time::sleep(Duration::from_secs_f32(seconds)).await;
}

pub mod window {
    use ambient_core::window::{window_ctl, WindowCtl};
    use ambient_ecs::World;
    use ambient_shared_types::CursorIcon;

    pub fn set_cursor(world: &World, cursor: CursorIcon) {
        world
            .resource(window_ctl())
            .send(WindowCtl::SetCursorIcon(cursor.into()))
            .ok();
    }

    pub async fn get_clipboard() -> Option<String> {
        ambient_sys::clipboard::get().await
    }

    pub async fn set_clipboard(text: &str) -> anyhow::Result<()> {
        ambient_sys::clipboard::set(text).await
    }
}
