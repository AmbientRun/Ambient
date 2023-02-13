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
