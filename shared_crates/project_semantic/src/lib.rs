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
    items: HashMap<Ulid, ItemValue>,
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

        let mut sem = Self {
            items: HashMap::new(),
            root_scope: Scope {
                id: Identifier::default(),
                manifest: None,
                scopes: BTreeMap::new(),
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
                types: BTreeMap::new(),
                attributes: BTreeMap::new(),
            },
            scopes: BTreeMap::new(),
        };

        for (id, ty) in primitive_component_definitions!(define_primitive_types) {
            let item_id = sem.add(ty);
            sem.root_scope.types.insert(id, item_id);
        }

        sem
    }

    pub fn add_file(
        &mut self,
        filename: &str,
        file_provider: &dyn FileProvider,
    ) -> anyhow::Result<&mut Scope> {
        let scope = Scope::from_file(self, filename, file_provider)?;
        Ok(self.scopes.entry(scope.id.clone()).or_insert(scope))
    }

    pub fn resolve(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn add<T: Item>(&mut self, item: T) -> ItemId<T> {
        let ulid = ulid::Ulid::new();
        self.items.insert(ulid, item.into_item_value());
        ItemId(ulid, PhantomData)
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
    components: BTreeMap<Identifier, ItemId<Component>>,
    concepts: BTreeMap<Identifier, ItemId<Concept>>,
    messages: BTreeMap<Identifier, ItemId<Message>>,
    types: BTreeMap<CamelCaseIdentifier, ItemId<Type>>,
    attributes: BTreeMap<Identifier, ItemId<Attribute>>,
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
    pub fn from_file(
        semantic: &mut Semantic,
        filename: &str,
        file_provider: &dyn FileProvider,
    ) -> anyhow::Result<Self> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let mut scopes = BTreeMap::new();
        for include in &manifest.project.includes {
            let scope = Scope::from_file(semantic, include, file_provider)?;
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
                .insert(last.clone(), semantic.add(component.into()));
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .concepts
                .insert(last.clone(), semantic.add(concept.into()));
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .messages
                .insert(last.clone(), semantic.add(message.into()));
        }

        for (segment, ty) in manifest.enums.iter() {
            scope.types.insert(segment.clone(), semantic.add(ty.into()));
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ItemType {
    Component,
    Concept,
    Message,
    Type,
    Attribute,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemValue {
    Component(Component),
    Concept(Concept),
    Message(Message),
    Type(Type),
    Attribute(Attribute),
}

pub trait Item {
    const TYPE: ItemType;
    type Unresolved: Eq + Debug;

    fn from_item_value(value: &ItemValue) -> Option<&Self>;
    fn into_item_value(self) -> ItemValue;
}

#[derive(Clone, Hash)]
pub struct ItemId<T: Item>(Ulid, PhantomData<T>);

impl<T: Item + Debug> Debug for ItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ItemId").field(&self.0 .0).finish()
    }
}
impl<T: Item> PartialEq for ItemId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Item> Eq for ItemId<T> {}
impl<T: Item> ItemId<T> {
    pub fn get<'a>(&'a self, semantic: &'a Semantic) -> Option<&'a T> {
        T::from_item_value(semantic.items.get(&self.0)?)
    }
}

#[derive(Clone)]
pub enum ItemRef<T: Item> {
    Unresolved(T::Unresolved),
    Resolved(ItemId<T>),
}
impl<T: Item + Debug> Debug for ItemRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(arg0) => write!(f, "Unresolved({arg0:?})"),
            Self::Resolved(arg0) => write!(f, "Resolved({arg0:?})"),
        }
    }
}
impl<T: Item> PartialEq for ItemRef<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unresolved(l0), Self::Unresolved(r0)) => l0 == r0,
            (Self::Resolved(l0), Self::Resolved(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl<T: Item> Eq for ItemRef<T> {}
impl<T: Item> std::hash::Hash for ItemRef<T> {
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
impl Item for Component {
    const TYPE: ItemType = ItemType::Component;
    type Unresolved = IdentifierPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Component(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Component(self)
    }
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
impl Item for Concept {
    const TYPE: ItemType = ItemType::Concept;
    type Unresolved = IdentifierPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Concept(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Concept(self)
    }
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
impl Item for Message {
    const TYPE: ItemType = ItemType::Message;
    type Unresolved = IdentifierPathBuf;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Message(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Message(self)
    }
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
impl Item for Attribute {
    const TYPE: ItemType = ItemType::Attribute;
    type Unresolved = CamelCaseIdentifier;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Attribute(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Attribute(self)
    }
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
impl Item for Type {
    const TYPE: ItemType = ItemType::Type;
    type Unresolved = ComponentType;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Type(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Type(self)
    }
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
