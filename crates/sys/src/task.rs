use std::{future::Future, task::Poll, time::Duration};

use derive_more::{Deref, From};
use futures::FutureExt;

use crate::{control::ControlHandle, platform};
pub use platform::task::wasm_nonsend;

/// Spawns a new background task
pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: 'static + Send + Future<Output = T>,
    T: 'static + Send,
{
    JoinHandle(platform::task::spawn(fut))
}

/// Spawn a non-send future by sending a constructor to a worker thread.
///
/// The future will run to completion on the worker thread.
///
/// Returns a handle which can be used to control the future
pub fn spawn_local<F, Fut, T>(func: F) -> ControlHandle<T>
where
    F: 'static + FnOnce() -> Fut + Send,
    Fut: 'static + Future<Output = T>,
    T: 'static + Send,
{
    platform::task::spawn_local(func)
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum JoinError {
    #[error("The future was aborted")]
    Aborted,
    #[error("The future panicked")]
    #[allow(dead_code)]
    Panicked,
}

pub struct JoinHandle<T>(pub(crate) platform::task::JoinHandle<T>);

impl<T> JoinHandle<T> {
    pub fn abort(&self) {
        self.0.abort()
    }

    /// Returns true if the task is currently finished or aborted
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

#[derive(From, Deref)]
pub struct AbortOnDrop<T>(JoinHandle<T>);

impl<T> Future for AbortOnDrop<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
