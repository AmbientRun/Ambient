pub use ambient_api_core as api;
pub use api::{core, message::*};

use std::future::Future;
pub fn run_async(_world: &ecs::World, future: impl Future<Output = ()> + Send + 'static) {
    api::prelude::run_async(async {
        future.await;
        api::prelude::OkEmpty
    });
}

/// Execute a future to completion on a worker thread.
///
/// This permits spawning thread local futures
pub fn run_async_local<F>(_world: &ecs::World, create: impl 'static + Send + FnOnce() -> F)
where
    F: 'static + Future,
    F::Output: Send + 'static,
{
    api::prelude::run_async(async {
        let future = create();
        future.await;
        api::prelude::OkEmpty
    });
}

pub async fn sleep(seconds: f32) {
    api::prelude::sleep(seconds).await;
}

pub mod ecs {
    use super::api;
    pub use api::{
        ecs::{Component, ECSError, SupportedValue as ComponentValue, UntypedComponent, World},
        prelude::{Entity, EntityId},
    };

    pub struct ComponentDesc(Box<dyn UntypedComponent>);
    impl ComponentDesc {
        pub fn index(&self) -> u32 {
            self.0.index()
        }
    }
    impl<T: 'static> From<Component<T>> for ComponentDesc {
        fn from(value: Component<T>) -> Self {
            Self(Box::new(value))
        }
    }
}

pub mod window {
    use ambient_shared_types::CursorIcon;

    pub fn set_cursor(_world: &crate::ecs::World, cursor: CursorIcon) {
        #[cfg(feature = "client")]
        super::api::client::input::set_cursor(cursor);
        #[cfg(not(feature = "client"))]
        let _ = cursor;
    }
    pub async fn get_clipboard() -> Option<String> {
        #[cfg(feature = "client")]
        return super::api::client::clipboard::get();
        #[cfg(not(feature = "client"))]
        return None;
    }

    pub async fn set_clipboard(_text: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Clipboard is not yet supported"))
    }
}
