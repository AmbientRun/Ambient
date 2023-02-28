//! At present, an entirely informational mechanism for describing groups of components that, when used together,
//! result in some behaviour.
//!
//! See the project manifest documentation for more information.

use crate::EntityData;

#[derive(Clone)]
pub struct Concept {
    pub id: String,
    pub name: String,
    pub description: String,
    pub extends: Vec<String>,
    pub data: EntityData,
}

pub struct RefConcept<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub extends: &'a [&'a str],
    pub data: EntityData,
}
impl<'a> RefConcept<'a> {
    pub fn to_owned(&self) -> Concept {
        Concept {
            id: self.id.to_owned(),
            name: self.name.to_owned(),
            description: self.description.to_owned(),
            extends: self.extends.iter().map(|s| s.to_string()).collect(),
            data: self.data.to_owned(),
        }
    }
}
