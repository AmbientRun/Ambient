/// An identifier for an entity in the world.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct EntityId {
    #[doc(hidden)]
    pub id0: u64,
    #[doc(hidden)]
    pub id1: u64,
}
impl EntityId {
    #[doc(hidden)]
    pub const fn new(id0: u64, id1: u64) -> Self {
        Self { id0, id1 }
    }
}
impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ((self.id0 as u128) << 64) + self.id1 as u128)
    }
}
impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
