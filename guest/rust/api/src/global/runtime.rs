use std::{cell::RefCell, future::Future, rc::Rc, task::Poll};

use crate::{
    global::EventResult,
    internal::{component::Components, executor::EXECUTOR, host},
};

/// The time, relative to when the application started, in seconds.
/// This can be used to time how long something takes.
pub fn time() -> f32 {
    EXECUTOR.frame_state().time()
}

/// The length of the previous frame, in seconds.
pub fn frametime() -> f32 {
    EXECUTOR.frame_state().frametime()
}

/// `on` calls `callback` every time `event` occurs.
///
/// If you only want to be notified once, use [once].
///
/// The `callback` is a `fn`. This can be a closure (e.g. `|args| { ... }`).
pub fn on(event: &str, callback: impl Fn(&Components) -> EventResult + 'static) {
    on_async(event, move |args| std::future::ready(callback(args)))
}

/// `on_async` calls `callback` every time `event` occurs.
///
/// If you only want to be notified once, use [once_async].
///
/// The `callback` is a `async fn`. This can be a closure (e.g. `|args| async move { ... }`).
pub fn on_async<R: Future<Output = EventResult> + 'static>(
    event: &str,
    callback: impl Fn(&Components) -> R + 'static,
) {
    host::event_subscribe(event);
    EXECUTOR.register_callback(
        event.to_string(),
        Box::new(move |args| Box::pin(callback(args))),
    );
}

/// `once` calls `callback` when `event` occurs, but only once.
///
/// If you want to be notified every time the `event` occurs, use [on].
///
/// The `callback` is a `fn`. This can be a closure (e.g. `|args| { ... }`).
pub fn once(event: &str, callback: impl FnOnce(&Components) -> EventResult + 'static) {
    once_async(event, |args| std::future::ready(callback(args)))
}

/// `once_async` calls `callback` when `event` occurs, but only once.
///
/// If you want to be notified every time the `event` occurs, use [on_async].
///
/// The `callback` is a `async fn`. This can be a closure (e.g. `|args| async move { ... }`).
pub fn once_async<R: Future<Output = EventResult> + 'static>(
    event: &str,
    callback: impl FnOnce(&Components) -> R + 'static,
) {
    host::event_subscribe(event);
    EXECUTOR.register_callback_once(
        event.to_string(),
        Box::new(move |args| Box::pin(callback(args))),
    );
}

/// Runs the given async block (`future`). This lets your module set up behaviour
/// to run concurrently, like a long-running task.
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
///     EventOk
/// });
/// ```
pub fn run_async(future: impl Future<Output = EventResult> + 'static) {
    EXECUTOR.spawn(Box::pin(future));
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
    let target_time = time() + seconds;
    block_until(|| time() > target_time).await
}

/// Stops execution of this function until `event` occurs with the specified `condition`.
/// Useful for waiting until a particular event has happened in the game world.
///
/// This must be used with `.await` in either an `async fn` or an `async` block.
pub async fn until_this(
    event: &str,
    condition: impl Fn(&Components) -> bool + 'static,
) -> Components {
    let ret = Rc::new(RefCell::new(None));

    fn register_callback(
        event: String,
        condition: impl Fn(&Components) -> bool + 'static,
        ret: Rc<RefCell<Option<Components>>>,
    ) {
        once(&event, {
            let event = event.clone();
            move |args: &Components| {
                if condition(args) {
                    let args = args.clone();
                    *ret.borrow_mut() = Some(args);
                } else {
                    register_callback(event, condition, ret);
                }
                Ok(())
            }
        });
    }
    register_callback(event.to_string(), condition, ret.clone());

    std::future::poll_fn(move |_cx| {
        ret.borrow_mut()
            .take()
            .map(Poll::Ready)
            .unwrap_or(Poll::Pending)
    })
    .await
}
