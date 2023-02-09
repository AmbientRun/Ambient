use std::{
    marker::Unsize, ops::{CoerceUnsized, Deref, DerefMut}, sync::Arc
};

/// This is just wrapping an Box, and it only exists because Box<dyn Fn..> doesn't implement Debug, so
/// we're wrapping it with a CbBox to avoid having to handle that in all structs that implement Debug
pub struct CbBox<T: ?Sized>(pub Box<T>);
impl<T> CbBox<T> {
    pub fn new(cb: T) -> Self {
        Self(Box::new(cb))
    }
}

impl<T: ?Sized> std::fmt::Debug for CbBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CbBox").finish()
    }
}

impl<T: ?Sized> Deref for CbBox<T> {
    type Target = <Arc<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl<T: ?Sized> DerefMut for CbBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

pub type Callback<T, Ret = ()> = Cb<dyn Fn(T) -> Ret + Sync + Send>;

/// This is just wrapping an Arc, and it only exists because Arc<dyn Fn..> doesn't implement Debug, so
/// we're wrapping it with a Cb to avoid having to handle that in all structs that implement Debug
#[derive(Default)]
pub struct Cb<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Cb<U>> for Cb<T> {}
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<CbBox<U>> for CbBox<T> {}

impl<T> Cb<T> {
    #[inline]
    pub fn new(val: T) -> Self {
        Self(Arc::new(val))
    }
}
impl<T: ?Sized> std::fmt::Debug for Cb<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cb").finish()
    }
}
impl<T: ?Sized> Clone for Cb<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: ?Sized> Deref for Cb<T> {
    type Target = <Arc<T> as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub fn log_error(err: &anyhow::Error) {
    #[cfg(feature = "sentry")]
    sentry_anyhow::capture_anyhow(err);
    #[cfg(not(feature = "sentry"))]
    tracing::error!("{:?}", err);
}

pub type CallbackFn<T, U = ()> = Arc<dyn Fn(T) -> U + Sync + Send + 'static>;
pub type CallbackBox<T, U = ()> = Box<dyn Fn(T) -> U + Sync + Send + 'static>;

pub type CellFn<T, U = ()> = dyn Fn(&mut T) -> U + Send + Sync;
pub type CellFnOnce<T, U = ()> = dyn Fn(&mut T) -> U + Send + Sync;
