use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use crate::{once, time, Components};

/// Stops execution of this function until the provided `condition` is true.
/// Useful for waiting for something to happen in the game world.
///
/// This must be used with `.await` in either an `async fn` or an `async` block.
pub async fn block_until(condition: impl Fn() -> bool) {
    struct BlockUntilFuture<F>(F);
    impl<F: Fn() -> bool> Future for BlockUntilFuture<F> {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.0() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }

    BlockUntilFuture(condition).await
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
pub async fn until_this(event: &str, condition: impl Fn(&Components) -> bool + Send + Sync + 'static) -> Components {
    let ret = Arc::new(Mutex::new(None));

    fn register_callback(
        event: String,
        condition: impl Fn(&Components) -> bool + Send + Sync + 'static,
        ret: Arc<Mutex<Option<Components>>>,
    ) {
        once(&event, {
            let event = event.clone();
            move |args: &Components| {
                if condition(args) {
                    let args = args.clone();
                    *ret.lock().unwrap() = Some(args);
                } else {
                    register_callback(event, condition, ret);
                }
                Ok(())
            }
        });
    }
    register_callback(event.to_string(), condition, ret.clone());

    struct WhenFuture<Args>(Arc<Mutex<Option<Args>>>);
    impl<Args> Future for WhenFuture<Args> {
        type Output = Args;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let is_ready = self.0.lock().unwrap().is_some();
            if is_ready {
                Poll::Ready(self.0.lock().unwrap().take().unwrap())
            } else {
                Poll::Pending
            }
        }
    }

    WhenFuture(ret).await
}
