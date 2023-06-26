use std::{future::Future, task::Poll, time::Duration};

use crate::{
    components, entity,
    global::{OkEmpty, ResultEmpty},
    internal::executor::EXECUTOR,
};

/// The time, relative to Jan 1, 1970.
pub fn time() -> Duration {
    entity::get_component(entity::resources(), components::core::app::absolute_time()).unwrap()
}

/// The length of the previous frame, in seconds.
pub fn delta_time() -> f32 {
    entity::get_component(entity::resources(), components::core::app::delta_time()).unwrap()
}

/// A trait that abstracts over return types so that you can return an [ResultEmpty] or nothing.
pub trait CallbackReturn {
    #[doc(hidden)]
    fn into_result(self) -> ResultEmpty;
}
impl CallbackReturn for ResultEmpty {
    fn into_result(self) -> ResultEmpty {
        self
    }
}
impl CallbackReturn for () {
    fn into_result(self) -> ResultEmpty {
        OkEmpty
    }
}

/// Runs the given async block (`future`). This lets your module set up behaviour
/// to run concurrently, like a long-running task. It can return either a [ResultEmpty] or
/// nothing.
///
/// This is similar to [tokio::spawn](https://docs.rs/tokio/latest/tokio/fn.spawn.html),
/// as well as similar functions from other async runtimes.
///
/// # Examples
/// ```
/// run_async(async {
///     notification::broadcast("a title", "hello!");
///     sleep(2.0).await;
///     notification::broadcast("a title", "hello to you too!");
/// });
/// ```
pub fn run_async<R: CallbackReturn>(future: impl Future<Output = R> + 'static) {
    EXECUTOR.spawn(Box::pin(async move { future.await.into_result() }));
}

/// Stops execution of this function until the provided `condition` is true.
/// Useful for waiting for something to happen in the game world.
///
/// This must be used with `.await` in either an `async fn` or an `async` block.
pub async fn block_until(condition: impl Fn() -> bool) {
    std::future::poll_fn(move |_cx| {
        if condition() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await
}

/// Stops execution of this function until `seconds` has passed.
///
/// This must be used with `.await` in either an `async fn` or an `async` block.
pub async fn sleep(seconds: f32) {
    let target_time = time() + Duration::from_secs_f32(seconds);
    block_until(|| time() > target_time).await
}
