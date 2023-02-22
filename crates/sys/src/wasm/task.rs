use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Poll, Waker},
    time::Duration,
};

use futures::{
    future::abortable,
    stream::{AbortHandle, Abortable},
};
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};

use crate::{
    control::{control_future, ControlHandle},
    task::JoinError,
};

/// Spawns a new background task
pub fn spawn<F, T>(fut: F) -> ControlHandle<T>
where
    F: 'static + Future<Output = T>,
    T: 'static,
{
    let (ctl, fut) = control_future(fut);

    wasm_bindgen_futures::spawn_local(fut);

    ctl
}

pub type JoinHandle<T> = ControlHandle<T>;

pub async fn wasm_nonsend<F, Fut, T>(func: F) -> T
where
    F: 'static + FnOnce() -> Fut + Send,
    Fut: 'static + Future<Output = T>,
    T: 'static + Send,
{
    spawn_local(func).await.unwrap()
}

pub fn spawn_local<F, Fut, T>(func: F) -> ControlHandle<T>
where
    F: 'static + FnOnce() -> Fut + Send,
    Fut: 'static + Future<Output = T>,
    T: 'static + Send,
{
    let (ctl, fut) = control_future(func());

    wasm_bindgen_futures::spawn_local(fut);

    ctl
}

#[inline]
pub async fn sleep(dur: Duration) {
    gloo::timers::future::sleep(dur).await
}

#[derive(Clone)]
pub struct RuntimeHandle {}

impl RuntimeHandle {
    pub fn spawn<F, T>(&self, fut: F) -> JoinHandle<T>
    where
        F: 'static + Future<Output = T>,
        T: 'static,
    {
        spawn(fut)
    }
}
