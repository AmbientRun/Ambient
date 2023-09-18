use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::Poll,
    time::{Duration, Instant},
};

use crate::{
    core::{app, messages::Frame},
    entity,
    global::{OkEmpty, ResultEmpty},
    internal::executor::EXECUTOR,
    message::Listener,
    prelude::RuntimeMessage,
};

/// The time, relative to the start of the game. Guaranteed to be monotonic.
pub fn game_time() -> Duration {
    entity::get_component(entity::resources(), app::components::game_time()).unwrap()
}

/// The time, relative to Jan 1, 1970. Not guaranteed to be monotonic. Use [game_time] for most applications.
pub fn epoch_time() -> Duration {
    entity::get_component(entity::resources(), app::components::epoch_time()).unwrap()
}

/// The length of the previous frame, in seconds.
pub fn delta_time() -> f32 {
    entity::get_component(entity::resources(), app::components::delta_time()).unwrap()
}

/// Runs `callback` at most every `dt`. The actual time taken is passed to `callback`.
///
/// Note that this is bound by the length of the tick, so if your environment is running
/// slower than `dt`, this will not run at the expected rate.
pub fn fixed_rate_tick(dt: Duration, mut callback: impl FnMut(Duration) + 'static) -> Listener {
    let mut last_tick = game_time();
    Frame::subscribe(move |_| {
        let delta = game_time() - last_tick;
        if delta < dt {
            return;
        }

        callback(delta);

        last_tick = game_time();
    })
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
    let target_time = Instant::now() + Duration::from_secs_f32(seconds);
    block_until(|| Instant::now() > target_time).await
}

/// Stops execution of this function until the given [`RuntimeMessage`] is received.
/// The `is_relevant` function is used to filter out messages that are not relevant.
///
/// This must be used with `.await` in either an `async fn` or an `async` block.
pub async fn wait_for_runtime_message<T: RuntimeMessage + Clone + 'static>(
    is_relevant: impl Fn(&T) -> bool + 'static,
) -> T {
    let result = Rc::new(RefCell::new(None));
    let mut listener = Some(T::subscribe({
        let result = result.clone();
        move |response| {
            if !is_relevant(&response) {
                return;
            }

            *result.borrow_mut() = Some(response);
        }
    }));

    std::future::poll_fn(move |_cx| match &*result.borrow() {
        Some(r) => {
            let r = (*r).clone();
            if let Some(listener) = listener.take() {
                listener.stop();
            }
            Poll::Ready(r)
        }
        _ => Poll::Pending,
    })
    .await
}

/// Stops execution of this function until one of the given [`RuntimeMessage`]sare received.
/// The `is_relevant` functions are used to filter out messages that are not relevant.
///
/// This must be used with `.await` in either an `async fn` or an `async` block.
// TODO: Can we do this with a race of `wait_for_runtime_message`? We would want the resulting
// future to cancel their subscriptions, which is why I haven't done it yet.
pub async fn wait_for_fallible_runtime_messages<
    Success: RuntimeMessage + Clone + 'static,
    Failure: RuntimeMessage + Clone + 'static,
>(
    is_relevant_success: impl Fn(&Success) -> bool + 'static,
    is_relevant_failure: impl Fn(&Failure) -> bool + 'static,
) -> Result<Success, Failure> {
    let result = Rc::new(RefCell::new(None));
    let mut success_listener = Some(Success::subscribe({
        let result = result.clone();
        move |response| {
            if !is_relevant_success(&response) {
                return;
            }

            *result.borrow_mut() = Some(Ok(response));
        }
    }));
    let mut failure_listener = Some(Failure::subscribe({
        let result = result.clone();
        move |response| {
            if !is_relevant_failure(&response) {
                return;
            }

            *result.borrow_mut() = Some(Err(response));
        }
    }));

    std::future::poll_fn(move |_cx| match &*result.borrow() {
        Some(r) => {
            let r = (*r).clone();
            if let Some(listener) = success_listener.take() {
                listener.stop();
            }
            if let Some(listener) = failure_listener.take() {
                listener.stop();
            }
            Poll::Ready(r)
        }
        _ => Poll::Pending,
    })
    .await
}
