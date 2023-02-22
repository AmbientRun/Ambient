use ambient_sys::time::Instant;
use std::{collections::HashSet, sync::Arc};

pub struct EventDispatcher<T: ?Sized> {
    handlers: HashSet<EventHandler<T>>,
    pub created_timestamp: Instant,
}

impl<T: ?Sized> EventDispatcher<T> {
    pub fn new() -> Self {
        Self { handlers: HashSet::new(), created_timestamp: Instant::now() }
    }
    pub fn new_with(handler: Arc<T>) -> Self {
        let mut s = Self::new();
        s.add(handler);
        s
    }
    pub fn add(&mut self, handler: Arc<T>) {
        self.handlers.insert(EventHandler(handler));
    }
    pub fn remove(&mut self, handler: Arc<T>) {
        self.handlers.remove(&EventHandler(handler));
    }
    pub fn iter(&self) -> impl Iterator<Item = Arc<T>> + '_ {
        self.handlers.iter().map(|x| x.0.clone())
    }
}

impl<T: ?Sized> Default for EventDispatcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Clone for EventDispatcher<T> {
    fn clone(&self) -> Self {
        Self { handlers: self.handlers.clone(), created_timestamp: self.created_timestamp }
    }
}

struct EventHandler<T: ?Sized>(Arc<T>);
impl<T: ?Sized> PartialEq for EventHandler<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Eq for EventHandler<T> {}
impl<T: ?Sized> std::hash::Hash for EventHandler<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state)
    }
}

impl<T: ?Sized> Clone for EventHandler<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
