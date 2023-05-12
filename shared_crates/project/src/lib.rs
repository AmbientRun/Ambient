mod component;
pub use component::*;
mod concept;
pub use concept::*;
mod identifier;
pub use identifier::*;
mod manifest;
pub use manifest::*;
mod version;
pub use version::*;
mod message;
pub use message::*;
mod enum_;
pub use enum_::*;

#[derive(serde::Deserialize, Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct TypeRef(pub String);
impl TypeRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
