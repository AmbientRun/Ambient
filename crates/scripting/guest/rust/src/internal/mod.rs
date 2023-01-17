pub(crate) mod async_helpers;
pub(crate) mod component;
pub(crate) mod conversion;
pub(crate) mod runtime;
pub(crate) mod shared;

/// An identifier for an entity in the world.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EntityId {
    #[doc(hidden)]
    pub namespace: u8,
    #[doc(hidden)]
    pub id: u64,
    #[doc(hidden)]
    pub gen: i32,
}
impl EntityId {
    #[doc(hidden)]
    pub const fn new(namespace: u8, id: u64, gen: i32) -> Self {
        Self { namespace, id, gen }
    }
}
impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.namespace, self.id, self.gen)
    }
}
impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// A unique identifier for an entity in the world. This will remain the same even across
/// undo/redo, unlike [EntityId].
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum EntityUid {
    #[doc(hidden)]
    Static(&'static str),
    #[doc(hidden)]
    Owned(String),
}
impl EntityUid {
    #[doc(hidden)]
    pub const fn new(id: &'static str) -> Self {
        Self::Static(id)
    }
}
impl AsRef<str> for EntityUid {
    fn as_ref(&self) -> &str {
        match self {
            EntityUid::Static(s) => s,
            EntityUid::Owned(s) => s.as_str(),
        }
    }
}
impl std::fmt::Display for EntityUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
impl std::fmt::Debug for EntityUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// An identifier for an object that can be spawned.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum ObjectRef {
    #[doc(hidden)]
    Static(&'static str),
    #[doc(hidden)]
    Owned(String),
}
impl ObjectRef {
    #[doc(hidden)]
    pub const fn new(id: &'static str) -> Self {
        Self::Static(id)
    }
}
impl AsRef<str> for ObjectRef {
    fn as_ref(&self) -> &str {
        match self {
            ObjectRef::Static(s) => s,
            ObjectRef::Owned(s) => s.as_str(),
        }
    }
}
