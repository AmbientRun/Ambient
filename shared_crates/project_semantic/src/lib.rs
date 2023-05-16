use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use ambient_project::{Identifier, IdentifierPathBuf, Manifest};
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context;
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

use ulid::Ulid;

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    scopes: HashMap<Identifier, Scope>,
}
impl Semantic {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    pub fn add_file(
        &mut self,
        filename: &str,
        file_provider: &dyn FileProvider,
    ) -> anyhow::Result<&mut Scope> {
        let scope = Scope::from_file(filename, file_provider)?;
        Ok(self.scopes.entry(scope.id.clone()).or_insert(scope))
    }
}

pub trait FileProvider {
    fn get(&self, filename: &str) -> std::io::Result<String>;
}

#[derive(Clone, PartialEq)]
pub struct Scope {
    id: Identifier,
    manifest: Manifest,
    children: HashMap<Identifier, Scope>,
}
impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("id", &self.id)
            .field("children", &self.children)
            .finish()
    }
}
impl Scope {
    pub fn from_file(filename: &str, file_provider: &dyn FileProvider) -> anyhow::Result<Self> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let mut children = HashMap::new();
        for include in &manifest.project.includes {
            let scope = Scope::from_file(include, file_provider)?;
            children.insert(scope.id.clone(), scope);
        }

        Ok(Scope {
            id: manifest.project.id.clone(),
            manifest,
            children,
        })
    }
}

pub enum ItemType {
    Component,
    Concept,
    Message,
    Type,
    Attribute,
}
pub trait IsItemType {
    const TYPE: ItemType;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrimitiveType {
    pub rust_type: String,
}
impl IsItemType for PrimitiveType {
    const TYPE: ItemType = ItemType::Type;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Vec(PrimitiveType),
    Option(PrimitiveType),
}
impl IsItemType for Type {
    const TYPE: ItemType = ItemType::Type;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ItemId<T: IsItemType>(Ulid, PhantomData<T>);

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ItemRef<T: IsItemType> {
    Unresolved(IdentifierPathBuf),
    Resolved(ItemId<T>),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Component {
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_: ItemRef<Type>,
    pub attributes: Vec<String>,
    pub default: Option<PrimitiveValue>,
}
impl IsItemType for Component {
    const TYPE: ItemType = ItemType::Component;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Concept {
    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Vec<ItemRef<Concept>>,
    pub components: BTreeMap<ItemRef<Component>, PrimitiveValue>,
}
impl IsItemType for Concept {
    const TYPE: ItemType = ItemType::Concept;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub description: Option<String>,
    pub fields: BTreeMap<Identifier, ItemRef<Type>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attribute {}
impl IsItemType for Attribute {
    const TYPE: ItemType = ItemType::Attribute;
}

macro_rules! define_primitive_value {
    ($(($value:ident, $type:ty)),*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum PrimitiveValue {
            $(
                $value($type),
            )*
        }
        $(
            impl From<$type> for PrimitiveValue {
                fn from(value: $type) -> Self {
                    Self::$value(value)
                }
            }
        )*
    };
}

pub type EntityId = u128;

primitive_component_definitions!(define_primitive_value);
