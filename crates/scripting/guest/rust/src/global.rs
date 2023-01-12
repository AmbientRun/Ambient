use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{Debug, Display},
    rc::Rc,
};

/// A helper for state that can be easily shared around callbacks.
pub struct State<T: ?Sized>(Rc<RefCell<T>>);
impl<T> State<T> {
    /// Creates a new `State` with the given `value`.
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    /// Immutably borrows the state. Use this to access the state without modifying it.
    pub fn borrow(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    /// Mutably borrows the state. Use this to modify the state.
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}
impl<T: Display> Display for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
    }
}
impl<T: Debug> Debug for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.borrow().fmt(f)
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
