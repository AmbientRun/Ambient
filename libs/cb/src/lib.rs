use std::{ops::Deref, sync::Arc};

pub type Callback<T, Ret = ()> = Cb<dyn Fn(T) -> Ret + Sync + Send>;

#[derive(Default)]
#[repr(transparent)]
pub struct CbDebuggable<T: ?Sized>(pub T);
impl<T: ?Sized> std::fmt::Debug for CbDebuggable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cb").finish()
    }
}
impl<T: ?Sized> Deref for CbDebuggable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Cb<T> = Arc<CbDebuggable<T>>;

/// Helper for constructing a `Cb`.
///
/// This is just wrapping an Arc, and it only exists because Arc<dyn Fn..> doesn't implement Debug, so
/// we're wrapping it with a Cb to avoid having to handle that in all structs that implement Debug
pub fn cb<T>(f: T) -> Cb<T> {
    Arc::new(CbDebuggable(f))
}

pub type CallbackFn<T, U = ()> = Cb<dyn Fn(T) -> U + Sync + Send + 'static>;
pub type CallbackBox<T, U = ()> = Box<dyn Fn(T) -> U + Sync + Send + 'static>;

pub type CellFn<T, U = ()> = dyn Fn(&mut T) -> U + Send + Sync;
pub type CellFnOnce<T, U = ()> = dyn Fn(&mut T) -> U + Send + Sync;
