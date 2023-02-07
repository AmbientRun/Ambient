use std::{
    fmt::{Debug, Display},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A helper for state that can be easily shared around callbacks.
pub struct State<T: ?Sized>(Arc<RwLock<T>>);
impl<T> State<T> {
    /// Creates a new `State` with the given `value`.
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    /// Immutably borrows the state. Use this to access the state without modifying it.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }

    /// Mutably borrows the state. Use this to modify the state.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }

    #[deprecated = "use State::read"]
    #[doc(hidden)]
    pub fn borrow(&self) -> RwLockReadGuard<'_, T> {
        self.read()
    }

    #[deprecated = "use State::write"]
    #[doc(hidden)]
    pub fn borrow_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.write()
    }
}
impl<T: Display> Display for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().fmt(f)
    }
}
impl<T: Debug> Debug for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().fmt(f)
    }
}
impl<T: Default> Default for State<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
impl<T: ?Sized> Clone for State<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
