use std::future::Future;

use crate::control::{control_future, ControlHandle};

pub type JoinHandle<T> = ControlHandle<T>;

#[inline(always)]
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

#[derive(Debug, Clone)]
pub struct RuntimeHandle;

impl RuntimeHandle {
    pub fn current() -> Self {
        RuntimeHandle
    }

    pub fn spawn<F, T>(&self, fut: F) -> JoinHandle<T>
    where
        F: 'static + Future<Output = T>,
        T: 'static,
    {
        let (ctl, fut) = control_future(fut);

        wasm_bindgen_futures::spawn_local(fut);

        ctl
    }

    pub fn block_in_place<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }

    /// Spawns a task such that blocking is accepted
    pub fn spawn_blocking<R, F>(&self, f: F) -> JoinHandle<R>
    where
        F: 'static + Send + FnOnce() -> R,
        R: 'static + Send,
    {
        self.spawn(async move { f() })
    }
}
