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

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ItemMap {
    items: HashMap<Ulid, ItemValue>,
    vec_items: HashMap<ItemId<Type>, ItemId<Type>>,
    option_items: HashMap<ItemId<Type>, ItemId<Type>>,
}
impl ItemMap {
    pub fn add<T: Item>(&mut self, item: T) -> ItemId<T> {
        let ulid = ulid::Ulid::new();
        self.items.insert(ulid, item.into_item_value());
        ItemId(ulid, PhantomData)
    }

    pub fn get<T: Item>(&self, id: ItemId<T>) -> Option<&T> {
        T::from_item_value(self.items.get(&id.0)?)
    }

    pub fn get_mut<T: Item>(&mut self, id: ItemId<T>) -> Option<&mut T> {
        T::from_item_value_mut(self.items.get_mut(&id.0)?)
    }

    pub fn insert<T: Item>(&mut self, id: ItemId<T>, item: T) {
        self.items.insert(id.0, item.into_item_value());
    }

    pub fn get_vec_id(&mut self, id: ItemId<Type>) -> ItemId<Type> {
        if let Some(id) = self.vec_items.get(&id).cloned() {
            return id;
        }

        let vec_id = self.add(Type::Vec(id));
        self.vec_items.insert(id, vec_id);

        vec_id
    }

    pub fn get_option_id(&mut self, id: ItemId<Type>) -> ItemId<Type> {
        if let Some(id) = self.option_items.get(&id).cloned() {
            return id;
        }

        let option_id = self.add(Type::Option(id));
        self.option_items.insert(id, option_id);

        option_id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    pub items: ItemMap,
    root_scope: Scope,
    pub scopes: BTreeMap<Identifier, Scope>,
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
            items: ItemMap::default(),
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
            let item_id = sem.items.add(ty);
            sem.root_scope.types.insert(id, item_id);
        }

        for name in [
            "Debuggable",
            "Networked",
            "Resource",
            "MaybeResource",
            "Store",
        ] {
            let id = CamelCaseIdentifier::new(name).unwrap();
            let item_id = sem.items.add(Attribute { id: id.clone() });
            sem.root_scope.attributes.insert(id, item_id);
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
        for scope in self.scopes.values_mut() {
            scope.resolve(&mut self.items, Scopes(vec![&self.root_scope]));
        }
        Ok(())
    }
}

pub trait FileProvider {
    fn get(&self, filename: &str) -> std::io::Result<String>;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Scopes<'a>(Vec<&'a Scope>);
impl<'a> Scopes<'a> {
    fn push(&mut self, scope: &'a Scope) {
        self.0.push(scope);
    }

    fn get_type_id(
        &self,
        items: &mut ItemMap,
        component_type: &ComponentType,
    ) -> Option<ItemId<Type>> {
        for scope in self.0.iter().rev() {
            match component_type {
                ComponentType::Identifier(id) => {
                    if let Some(id) = scope.get_type_id(id) {
                        return Some(id);
                    }
                }
                ComponentType::ContainerType {
                    type_,
                    element_type,
                } => {
                    if let Some(id) = scope.get_type_id(element_type) {
                        return Some(match type_ {
                            ambient_project::ContainerType::Vec => items.get_vec_id(id),
                            ambient_project::ContainerType::Option => items.get_option_id(id),
                        });
                    }
                }
            }
        }
        None
    }

    fn get_attribute_id(&self, path: &CamelCaseIdentifier) -> Option<ItemId<Attribute>> {
        for scope in self.0.iter().rev() {
            if let Some(id) = scope.get_attribute_id(path) {
                return Some(id);
            }
        }
        None
    }
}

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub id: Identifier,
    manifest: Option<Manifest>,
    pub scopes: BTreeMap<Identifier, Scope>,

    pub components: BTreeMap<Identifier, ItemId<Component>>,
    pub concepts: BTreeMap<Identifier, ItemId<Concept>>,
    pub messages: BTreeMap<Identifier, ItemId<Message>>,
    pub types: BTreeMap<CamelCaseIdentifier, ItemId<Type>>,
    pub attributes: BTreeMap<CamelCaseIdentifier, ItemId<Attribute>>,
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
                .insert(last.clone(), semantic.items.add(component.into()));
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .concepts
                .insert(last.clone(), semantic.items.add(concept.into()));
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (parent, last) = path.parent_and_last();
            scope
                .get_scope_mut(parent)
                .messages
                .insert(last.clone(), semantic.items.add(message.into()));
        }

        for (segment, ty) in manifest.enums.iter() {
            scope.types.insert(
                segment.clone(),
                semantic
                    .items
                    .add(Type::from_project_enum(segment.clone(), ty)),
            );
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

    fn resolve<'a>(&'a self, items: &mut ItemMap, mut scopes: Scopes<'a>) {
        fn resolve<T: Item, U>(
            item_ids: &BTreeMap<U, ItemId<T>>,
            items: &mut ItemMap,
            scopes: &Scopes,
        ) {
            for id in item_ids.values().copied() {
                let item = items.get(id).cloned().unwrap().resolve(items, scopes);
                items.insert(id, item);
            }
        }

        scopes.push(self);

        resolve(&self.components, items, &scopes);
        resolve(&self.concepts, items, &scopes);
        resolve(&self.messages, items, &scopes);
        resolve(&self.types, items, &scopes);
        resolve(&self.attributes, items, &scopes);

        for scope in self.scopes.values() {
            scope.resolve(items, scopes.clone());
        }
    }

    fn get_type_id(&self, id: &CamelCaseIdentifier) -> Option<ItemId<Type>> {
        self.types.get(id).copied()
    }

    fn get_attribute_id(&self, path: &CamelCaseIdentifier) -> Option<ItemId<Attribute>> {
        self.attributes.get(path).copied()
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

pub trait Item: Clone {
    const TYPE: ItemType;
    type Unresolved: Eq + Debug;

    fn from_item_value(value: &ItemValue) -> Option<&Self>;
    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self>;
    fn into_item_value(self) -> ItemValue;
    fn resolve(&mut self, items: &mut ItemMap, scopes: &Scopes) -> Self;
}

pub struct ItemId<T: Item>(Ulid, PhantomData<T>);
impl<T: Item> std::hash::Hash for ItemId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
impl<T: Item> Copy for ItemId<T> {}
impl<T: Item> Clone for ItemId<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
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

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Component(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Component(self)
    }

    fn resolve(&mut self, items: &mut ItemMap, scopes: &Scopes) -> Self {
        let mut new = self.clone();

        new.type_ = match new.type_ {
            ItemRef::Unresolved(path) => {
                let id = scopes.get_type_id(items, &path).unwrap();
                ItemRef::Resolved(id)
            }
            t => t,
        };

        let mut attributes = vec![];
        for attribute in &new.attributes {
            attributes.push(match attribute {
                ItemRef::Unresolved(path) => {
                    let id = scopes.get_attribute_id(&path).unwrap();
                    ItemRef::Resolved(id)
                }
                t => t.clone(),
            });
        }
        new.attributes = attributes;

        new
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

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Concept(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Concept(self)
    }

    fn resolve(&mut self, items: &mut ItemMap, scopes: &Scopes) -> Self {
        self.clone()
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

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Message(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Message(self)
    }

    fn resolve(&mut self, items: &mut ItemMap, scopes: &Scopes) -> Self {
        self.clone()
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
pub struct Attribute {
    pub id: CamelCaseIdentifier,
}
impl Item for Attribute {
    const TYPE: ItemType = ItemType::Attribute;
    type Unresolved = CamelCaseIdentifier;

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Attribute(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Attribute(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Attribute(self)
    }

    fn resolve(&mut self, items: &mut ItemMap, scopes: &Scopes) -> Self {
        self.clone()
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
pub struct Enum {
    id: CamelCaseIdentifier,
    members: Vec<EnumMember>,
}

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
    Vec(ItemId<Type>),
    Option(ItemId<Type>),
    Enum(Enum),
}
impl Type {
    fn from_project_enum(id: CamelCaseIdentifier, value: &ambient_project::Enum) -> Self {
        Self::Enum(Enum {
            id,
            members: value.0.iter().map(|v| v.into()).collect(),
        })
    }

    pub fn to_string(&self, semantic: &Semantic) -> String {
        match self {
            Type::Primitive(pt) => pt.rust_type.clone(),
            Type::Vec(id) => {
                let inner = semantic.items.get(*id).unwrap();
                format!("Vec<{}>", inner.to_string(semantic))
            }
            Type::Option(id) => {
                let inner = semantic.items.get(*id).unwrap();
                format!("Option<{}>", inner.to_string(semantic))
            }
            Type::Enum(e) => e.id.to_string(),
        }
    }
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

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Type(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Type(self)
    }

    fn resolve(&mut self, items: &mut ItemMap, scopes: &Scopes) -> Self {
        self.clone()
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
