use futures::{future::LocalBoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use once_cell::sync::Lazy;
use std::{future::Future, pin::Pin, task::Poll};

use crate::{
    control::{control_deferred, ControlHandle},
    task::JoinError,
};

pub struct JoinHandle<T>(pub(crate) tokio::task::JoinHandle<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for JoinHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<tokio::task::JoinHandle<T>> for crate::task::JoinHandle<T> {
    fn from(value: tokio::task::JoinHandle<T>) -> Self {
        Self(JoinHandle(value))
    }
}

type NonSendCons = Box<dyn FnOnce() -> LocalBoxFuture<'static, ()> + Send>;

static LOCAL_WORKER: Lazy<flume::Sender<NonSendCons>> = Lazy::new(|| {
    // Create a new thread which runs the local futures
    let (tx, rx) = flume::unbounded::<NonSendCons>();
    let mut rx = rx.into_stream();

    tokio::task::spawn_blocking(|| {
        let rt = tokio::runtime::Handle::current();
        let mut set = FuturesUnordered::new();
        rt.block_on(async move {
            loop {
                tokio::select! {
                    Some(()) = set.next() => {
                        tracing::info!("Local future completed");
                    },
                    Some(task) = rx.next() => {
                        tracing::info!("Received new future");
                        set.push(task());
                    },
                }
            }
        });
    });

    tx
});

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
    ) -> Poll<Self::Output> {
        match self.0.poll_unpin(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(value)),
            Poll::Ready(Err(err)) if err.is_panic() => Poll::Ready(Err(JoinError::Panicked)),
            Poll::Ready(Err(err)) if err.is_cancelled() => Poll::Ready(Err(JoinError::Aborted)),
            Poll::Ready(Err(_)) => unreachable!(),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub fn spawn_local<F, Fut, T>(func: F) -> ControlHandle<T>
where
    F: 'static + FnOnce() -> Fut + Send,
    Fut: 'static + Future<Output = T>,
    T: 'static + Send,
{
    let (ctl, reg) = control_deferred();

    LOCAL_WORKER
        .send(Box::new(move || Box::pin(reg.control(func()))))
        .expect("Worker thread terminated");

    ctl
}

#[inline(always)]
/// Wraps a constructor function to send and construct the future on a worker thread
pub async fn wasm_nonsend<F, Fut, T>(func: F) -> T
where
    F: 'static + FnOnce() -> Fut + Send,
    Fut: 'static + Future<Output = T>,
    T: 'static + Send,
{
    func().await
}

#[derive(Debug, Clone)]
pub struct RuntimeHandle(tokio::runtime::Handle);

impl RuntimeHandle {
    #[inline]
    pub fn current() -> Self {
        Self(tokio::runtime::Handle::current())
    }

    /// Spawns a new background task
    #[inline]
    pub fn spawn<F, T>(&self, fut: F) -> JoinHandle<T>
    where
        F: 'static + Send + Future<Output = T>,
        T: 'static + Send,
    {
        JoinHandle(self.0.spawn(fut))
    }
    pub fn block_in_place<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = self.0.enter();
        tokio::task::block_in_place(f)
    }

    pub fn spawn_blocking<R, F>(&self, f: F) -> JoinHandle<R>
    where
        F: 'static + Send + FnOnce() -> R,
        R: 'static + Send,
    {
        JoinHandle(self.0.spawn_blocking(f))
    }
}

impl From<tokio::runtime::Handle> for crate::task::RuntimeHandle {
    fn from(value: tokio::runtime::Handle) -> Self {
        Self(RuntimeHandle(value))
    }
}

pub(crate) struct PlatformBoxFutureImpl<T>(Pin<Box<dyn Future<Output = T> + Send>>);

impl<T> PlatformBoxFutureImpl<T> {
    pub fn from_boxed(fut: Pin<Box<dyn Future<Output = T> + Send>>) -> Self {
        Self(fut)
    }

    #[inline]
    pub fn into_shared(self) -> Pin<Box<dyn Future<Output = T> + Send>> {
        self.0
    }
}

impl<T> Future for PlatformBoxFutureImpl<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}
