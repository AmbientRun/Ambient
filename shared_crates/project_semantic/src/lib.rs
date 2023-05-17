use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    marker::PhantomData,
};

use ambient_project::{
    CamelCaseIdentifier, ComponentType, Identifier, IdentifierPath, IdentifierPathBuf, Manifest,
};
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context;
use glam::{Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

use thiserror::Error;
use ulid::Ulid;

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    root_scope: Scope,
    scopes: BTreeMap<Identifier, Scope>,
}
impl Semantic {
    pub fn new() -> Self {
        macro_rules! define_primitive_types {
            ($(($value:ident, $type:ty)),*) => {
                [
                    $(
                        (
                            CamelCaseIdentifier::new(stringify!($value)).unwrap(),
                            Type::Primitive(PrimitiveType {
                                rust_type: stringify!($type).to_string(),
                            }),
                        )
                    ),*
                ]
            };
        }

        Self {
            root_scope: Scope {
                id: Identifier::default(),
                manifest: None,
                scopes: BTreeMap::new(),
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
                types: BTreeMap::from_iter(primitive_component_definitions!(
                    define_primitive_types
                )),
                attributes: BTreeMap::new(),
            },
            scopes: BTreeMap::new(),
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

    pub fn resolve(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub trait FileProvider {
    fn get(&self, filename: &str) -> std::io::Result<String>;
}

#[derive(Clone, PartialEq)]
pub struct Scope {
    id: Identifier,
    manifest: Option<Manifest>,

    scopes: BTreeMap<Identifier, Scope>,
    components: BTreeMap<Identifier, Component>,
    concepts: BTreeMap<Identifier, Concept>,
    messages: BTreeMap<Identifier, Message>,
    types: BTreeMap<CamelCaseIdentifier, Type>,
    attributes: BTreeMap<Identifier, Attribute>,
}
impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("Scope");
        ds.field("id", &self.id);

        if !self.components.is_empty() {
            ds.field("components", &self.components);
        }
        if !self.concepts.is_empty() {
            ds.field("concepts", &self.concepts);
        }
        if !self.messages.is_empty() {
            ds.field("messages", &self.messages);
        }
        if !self.types.is_empty() {
            ds.field("types", &self.types);
        }
        if !self.attributes.is_empty() {
            ds.field("attributes", &self.attributes);
        }
        if !self.scopes.is_empty() {
            ds.field("scopes", &self.scopes);
        }

        ds.finish()
    }
}
impl Scope {
    pub fn from_file(filename: &str, file_provider: &dyn FileProvider) -> anyhow::Result<Self> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let mut scopes = BTreeMap::new();
        for include in &manifest.project.includes {
            let scope = Scope::from_file(include, file_provider)?;
            scopes.insert(scope.id.clone(), scope);
        }

        let mut scope = Scope {
            id: manifest.project.id.clone(),
            manifest: None,
            scopes,

            components: BTreeMap::new(),
            concepts: BTreeMap::new(),
            messages: BTreeMap::new(),
            types: BTreeMap::new(),
            attributes: BTreeMap::new(),
        };

        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .components
                .insert(last.clone(), component.into());
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .concepts
                .insert(last.clone(), concept.into());
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .messages
                .insert(last.clone(), message.into());
        }

        for (segment, ty) in manifest.enums.iter() {
            scope.types.insert(segment.clone(), ty.into());
        }

        scope.manifest = Some(manifest);

        Ok(scope)
    }

    fn get_scope_mut(&mut self, path: IdentifierPath) -> &mut Scope {
        let mut scope = self;
        for segment in path.iter() {
            scope = scope
                .scopes
                .entry(segment.clone())
                .or_insert_with(|| Scope {
                    id: segment.clone(),
                    manifest: None,
                    scopes: Default::default(),
                    components: Default::default(),
                    concepts: Default::default(),
                    messages: Default::default(),
                    types: Default::default(),
                    attributes: Default::default(),
                });
        }
        scope
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
    type Unresolved: Eq + Debug;
}

#[derive(Clone, Debug, Hash)]
pub struct ItemId<T: IsItemType>(Ulid, PhantomData<T>);
impl<T: IsItemType> PartialEq for ItemId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: IsItemType> Eq for ItemId<T> {}

#[derive(Clone)]
pub enum ItemRef<T: IsItemType> {
    Unresolved(T::Unresolved),
    Resolved(ItemId<T>),
}
impl<T: IsItemType + Debug> Debug for ItemRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(arg0) => write!(f, "Unresolved({arg0:?})"),
            Self::Resolved(arg0) => write!(f, "Resolved({arg0:?})"),
        }
    }
}
impl<T: IsItemType> PartialEq for ItemRef<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unresolved(l0), Self::Unresolved(r0)) => l0 == r0,
            (Self::Resolved(l0), Self::Resolved(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl<T: IsItemType> Eq for ItemRef<T> {}
impl<T: IsItemType> std::hash::Hash for ItemRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum MaybePrimitiveValue {
    Unresolved(toml::Value),
    Resolved(PrimitiveValue),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Component {
    pub name: Option<String>,
    pub description: Option<String>,
    pub type_: ItemRef<Type>,
    pub attributes: Vec<ItemRef<Attribute>>,
    pub default: Option<MaybePrimitiveValue>,
}
impl IsItemType for Component {
    const TYPE: ItemType = ItemType::Component;
    type Unresolved = IdentifierPathBuf;
}
impl From<&ambient_project::Component> for Component {
    fn from(value: &ambient_project::Component) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.clone(),
            type_: ItemRef::Unresolved(value.type_.clone()),
            attributes: value
                .attributes
                .iter()
                .map(|attribute| ItemRef::Unresolved(attribute.clone()))
                .collect(),
            default: value
                .default
                .as_ref()
                .map(|v| MaybePrimitiveValue::Unresolved(v.clone())),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Concept {
    pub name: Option<String>,
    pub description: Option<String>,
    pub extends: Vec<ItemRef<Concept>>,
    pub components: HashMap<ItemRef<Component>, MaybePrimitiveValue>,
}
impl IsItemType for Concept {
    const TYPE: ItemType = ItemType::Concept;
    type Unresolved = IdentifierPathBuf;
}
impl From<&ambient_project::Concept> for Concept {
    fn from(value: &ambient_project::Concept) -> Self {
        Concept {
            name: value.name.clone(),
            description: value.description.clone(),
            extends: value
                .extends
                .iter()
                .map(|v| ItemRef::Unresolved(v.clone()))
                .collect(),
            components: value
                .components
                .iter()
                .map(|(k, v)| {
                    (
                        ItemRef::Unresolved(k.clone()),
                        MaybePrimitiveValue::Unresolved(v.clone()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub description: Option<String>,
    pub fields: BTreeMap<Identifier, ItemRef<Type>>,
}
impl IsItemType for Message {
    const TYPE: ItemType = ItemType::Message;
    type Unresolved = IdentifierPathBuf;
}
impl From<&ambient_project::Message> for Message {
    fn from(value: &ambient_project::Message) -> Self {
        Message {
            description: value.description.clone(),
            fields: value
                .fields
                .iter()
                .map(|(k, v)| (k.clone(), ItemRef::Unresolved(v.clone())))
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attribute {}
impl IsItemType for Attribute {
    const TYPE: ItemType = ItemType::Attribute;
    type Unresolved = CamelCaseIdentifier;
}

#[derive(Clone, PartialEq, Eq)]
pub struct PrimitiveType {
    pub rust_type: String,
}
impl Debug for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrimitiveType({:?})", self.rust_type)
    }
}
impl IsItemType for PrimitiveType {
    const TYPE: ItemType = ItemType::Type;
    type Unresolved = CamelCaseIdentifier;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Enum(Vec<EnumMember>);

#[derive(Clone, PartialEq, Eq)]
pub struct EnumMember {
    pub name: CamelCaseIdentifier,
    pub description: Option<String>,
}

impl Debug for EnumMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.name, self.description)
    }
}
impl From<&ambient_project::EnumMember> for EnumMember {
    fn from(value: &ambient_project::EnumMember) -> Self {
        Self {
            name: value.name.clone(),
            description: value.description.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Primitive(PrimitiveType),
    Vec(PrimitiveType),
    Option(PrimitiveType),
    Enum(Enum),
}
impl IsItemType for Type {
    const TYPE: ItemType = ItemType::Type;
    type Unresolved = ComponentType;
}
impl From<&ambient_project::Enum> for Type {
    fn from(value: &ambient_project::Enum) -> Self {
        Self::Enum(Enum(value.0.iter().map(|v| v.into()).collect()))
    }
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

#[allow(dead_code)]
#[derive(Error, Debug)]
enum ParseError {
    #[error("failed to parse toml value {0:?}")]
    TomlParseError(toml::Value),
}
