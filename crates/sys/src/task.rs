use std::{future::Future, pin::Pin, task::Poll};

use derive_more::{Deref, From};
use futures::FutureExt;

use crate::{control::ControlHandle, platform};
pub use platform::task::wasm_nonsend;

#[cfg(not(target_os = "unknown"))]
pub use platform::task::make_native_multithreaded_runtime;

/// Spawns a new background task in the current runtime
pub fn spawn<F, T>(fut: F) -> JoinHandle<T>
where
    F: 'static + Send + Future<Output = T>,
    T: 'static + Send,
{
    RuntimeHandle::current().spawn(fut)
}

/// Runs a blocking function without blocking the executor.
///
/// Kind of... it is a little bit of a lie as blocking in general is not possible so this will
/// simply run in place on wasm.
pub fn block_in_place<R, F>(f: F) -> R
where
    F: FnOnce() -> R,
{
    RuntimeHandle::current().block_in_place(f)
}

/// Spawns a task such that blocking is accepted
pub fn spawn_blocking<R, F>(f: F) -> JoinHandle<R>
where
    F: 'static + Send + FnOnce() -> R,
    R: 'static + Send,
{
    RuntimeHandle::current().spawn_blocking(f)
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

/// A handle for (optionally) joining a spawned task.
///
/// Dropping a JoinHandle does *not* cancel the task.
pub struct JoinHandle<T>(pub(crate) platform::task::JoinHandle<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for JoinHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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
/// Represents a children task that will cancel when dropped.
pub struct ChildTask<T>(JoinHandle<T>);

impl<T> Future for ChildTask<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

impl<T> Drop for ChildTask<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[derive(Debug, Clone, From)]
/// Represents a handle to the platform specific runtime
pub struct RuntimeHandle(pub(crate) platform::task::RuntimeHandle);

impl RuntimeHandle {
    #[inline]
    pub fn current() -> Self {
        Self(platform::task::RuntimeHandle::current())
    }

    /// Spawns a task
    #[inline]
    pub fn spawn<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: 'static + Send + Future<Output = T>,
        T: 'static + Send,
    {
        JoinHandle(self.0.spawn(future))
    }

    #[inline]
    #[cfg(target_os = "unknown")]
    pub fn spawn_local<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: 'static + Future<Output = T>,
        T: 'static,
    {
        JoinHandle(self.0.spawn(future))
    }

    pub fn block_in_place<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.0.block_in_place(f)
    }

    /// Spawns a task such that blocking is accepted
    pub fn spawn_blocking<R, F>(&self, f: F) -> JoinHandle<R>
    where
        F: 'static + Send + FnOnce() -> R,
        R: 'static + Send,
    {
        JoinHandle(self.0.spawn_blocking(f))
    }

    /// Runs a future to completion on the current thread using this runtime
    ///
    /// This is only available on the native runtime, where Tokio is present and can be
    /// used to run the future on the current thread.
    ///
    /// There is no equivalent operation for WASM.
    #[cfg(not(target_os = "unknown"))]
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
}

#[must_use = "Futures do nothing if not polled"]
pub struct PlatformBoxFuture<'a, T>(platform::task::PlatformBoxFutureImpl<'a, T>);

impl<'a, T> PlatformBoxFuture<'a, T> {
    #[cfg(target_os = "unknown")]
    pub fn new<F>(future: F) -> Self
    where
        F: 'a + Future<Output = T>,
    {
        Self(platform::task::PlatformBoxFutureImpl::from_boxed(Box::pin(
            future,
        )))
    }

    #[cfg(not(target_os = "unknown"))]
    pub fn new<F>(future: F) -> Self
    where
        F: 'a + Future<Output = T> + Send,
    {
        Self(platform::task::PlatformBoxFutureImpl::from_boxed(Box::pin(
            future,
        )))
    }

    #[cfg(target_os = "unknown")]
    /// Convert this into a thread local future for [`wasm_bindgen_futures::spawn_local`] or
    /// [`tokio::LocalSet`].
    pub fn into_local(self) -> futures::future::LocalBoxFuture<'a, T> {
        self.0.into_local()
    }

    #[cfg(not(target_os = "unknown"))]
    /// Convert into a sendable future, sutable for [`tokio::spawn`].
    pub fn into_shared(self) -> futures::future::BoxFuture<'a, T> {
        self.0.into_shared()
    }
}

impl<'a, T> Future for PlatformBoxFuture<'a, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}
