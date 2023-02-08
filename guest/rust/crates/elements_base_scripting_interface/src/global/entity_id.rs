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
