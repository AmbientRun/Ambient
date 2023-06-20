pub use ambient_ecs as ecs;
use std::{future::Future, time::Duration};

pub use ecs::{
    generated::{components::core as components, concepts, messages},
    Message, RuntimeMessage,
};

pub fn run_async(world: &ecs::World, future: impl Future<Output = ()> + Send + 'static) {
    world.resource(ambient_core::runtime()).spawn(future);
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
    #[cfg(not(target_os = "unknown"))]
    pub fn get_clipboard() -> Option<String> {
        ambient_sys::clipboard::get()
    }

    #[cfg(target_os = "unknown")]
    pub async fn get_clipboard() -> Option<String> {
        ambient_sys::clipboard::get().await
    }

    #[cfg(target_os = "unknown")]
    pub async fn set_clipboard(text: &str) -> Result<(), String> {
        ambient_sys::clipboard::set(text).await
    }
}
