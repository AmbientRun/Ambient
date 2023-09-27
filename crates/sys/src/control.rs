use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Poll, Waker},
};

use futures::{task::AtomicWaker, Future};
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};

use crate::task::JoinError;

struct InnerState<T> {
    task_waker: AtomicWaker,
    handle_waker: AtomicWaker,
    res: Mutex<Option<Result<T, JoinError>>>,
    woken: AtomicBool,
    aborted: AtomicBool,
}

/// A registered control interface which will allow a future to be controlled.
pub struct ControlRegistration<T> {
    inner: Arc<InnerState<T>>,
}

impl<T> ControlRegistration<T> {
    fn new() -> Self {
        Self {
            inner: Arc::new(InnerState::new()),
        }
    }

    /// Take control of this future
    pub fn control<F>(self, fut: F) -> ControlledFuture<F, T> {
        ControlledFuture {
            fut,
            state: self.inner,
        }
    }
}

impl<T> Default for ControlRegistration<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InnerState<T> {
    fn new() -> Self {
        Self {
            task_waker: AtomicWaker::new(),
            handle_waker: AtomicWaker::new(),
            res: Mutex::new(None),
            woken: AtomicBool::new(false),
            aborted: AtomicBool::new(false),
        }
    }

    fn wake_handle(&self) {
        // Set woken regardless of waker, since the task can complete before the JoinHandle is
        // polled
        self.woken.store(true, Ordering::SeqCst);

        self.handle_waker.wake();
    }
}

#[pin_project(PinnedDrop)]
/// A future which is controlled elsewhere
pub struct ControlledFuture<F, T> {
    #[pin]
    fut: F,
    state: Arc<InnerState<T>>,
}

#[pinned_drop]
impl<F, T> PinnedDrop for ControlledFuture<F, T> {
    fn drop(self: Pin<&mut Self>) {
        let mut res = self.state.res.lock();
        if res.is_none() {
            // Cancelled on behalf of the executor
            *res = Some(Err(JoinError::Aborted));

            self.state.wake_handle();
        }
    }
}

impl<F, T> Future for ControlledFuture<F, T>
where
    F: Future<Output = T>,
{
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let p = self.project();

        p.state.task_waker.register(cx.waker());

        if p.state.aborted.load(Ordering::SeqCst) {
            tracing::info!("Future aborted");
            let mut res = p.state.res.lock();
            *res = Some(Err(JoinError::Aborted));

            p.state.wake_handle();
            Poll::Ready(())
        } else if let Poll::Ready(value) = p.fut.poll(cx) {
            let mut res = p.state.res.lock();
            assert!(res.is_none(), "Future completed twice");
            *res = Some(Ok(value));

            p.state.wake_handle();

            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// Represents a handle for controlling another future and awaiting its result.
///
/// **Note**: awaiting the handle does **not** progress the controlled future. The future must be
/// polled separately through e.g; spawning.
pub struct ControlHandle<T> {
    state: Arc<InnerState<T>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for ControlHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControlHandle")
            .field("result", &self.state.res)
            .field("aborted", &self.state.aborted.load(Ordering::Relaxed))
            .finish()
    }
}

impl<T> ControlHandle<T> {
    /// Remotely cancel the future
    pub fn abort(&self) {
        tracing::debug!("aborting controlled task");
        self.state.aborted.store(true, Ordering::SeqCst);

        self.state.task_waker.wake();
    }

    /// Returns true if the controlled future is currently finished or aborted
    pub fn is_finished(&self) -> bool {
        self.state.res.lock().is_some()
    }
}

impl<T> Future for ControlHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if self
            .state
            .woken
            .compare_exchange(true, false, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            eprintln!("Future completed");

            let value = self.state.res.lock().take().unwrap();
            Poll::Ready(value)
        } else {
            // Set a waker
            self.state.handle_waker.register(cx.waker());
            Poll::Pending
        }
    }
}

/// Allows a future to be remotely controlled
pub fn control_future<F, T>(fut: F) -> (ControlHandle<T>, ControlledFuture<F, T>)
where
    F: Future<Output = T>,
    T: 'static,
{
    let reg = ControlRegistration::new();

    (
        ControlHandle {
            state: reg.inner.clone(),
        },
        reg.control(fut),
    )
}

/// Obtain a control handle and a permit which allows you to associate and control a future with
/// the returned [`ControlHandle`].
pub fn control_deferred<T>() -> (ControlHandle<T>, ControlRegistration<T>)
where
    T: 'static,
{
    let reg = ControlRegistration::new();

    (
        ControlHandle {
            state: reg.inner.clone(),
        },
        reg,
    )
}
