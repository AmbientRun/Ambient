use std::{fmt::Debug, ops::Deref, sync::Arc};

use parking_lot::{Mutex, MutexGuard};

/// Represents an abstraction over constants or shared values, allowing sources to use both
/// constants, and `Arc<Mutex<V>>` interchangeably.
pub trait Value<'a>: Send {
    type Item;
    type Guard: Deref<Target = Self::Item>;
    fn get(&'a self) -> Self::Guard;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Constant<T>(pub T);

impl<'a, T> Value<'a> for Constant<T>
where
    T: 'a + Send,
{
    type Item = T;
    type Guard = &'a T;

    fn get(&'a self) -> Self::Guard {
        &self.0
    }
}

impl<'a, T> Value<'a> for Mutex<T>
where
    T: 'a + Send,
{
    type Item = T;
    type Guard = MutexGuard<'a, T>;

    fn get(&'a self) -> Self::Guard {
        self.lock()
    }
}

impl<'a, T> Value<'a> for Arc<T>
where
    T: Send + Sync + Value<'a>,
{
    type Item = T::Item;
    type Guard = T::Guard;

    fn get(&'a self) -> Self::Guard {
        self.deref().get()
    }
}
