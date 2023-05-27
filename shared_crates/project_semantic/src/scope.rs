use ambient_project::{ComponentType, Identifier, ItemPath, Manifest};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Attribute, Component, Concept, FileProvider, Item, ItemId, ItemMap, ItemType, ItemValue,
    Message, Resolve, ResolveClone, Semantic, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Context(Vec<ItemId<Scope>>);
impl Context {
    pub(crate) fn new(root_scope: ItemId<Scope>) -> Self {
        Self(vec![root_scope])
    }

    fn push(&mut self, scope: ItemId<Scope>) {
        self.0.push(scope);
    }

    pub(crate) fn get_type_id(
        &self,
        items: &ItemMap,
        component_type: &ComponentType,
    ) -> Option<ItemId<Type>> {
        for &scope_id in self.0.iter().rev() {
            match component_type {
                ComponentType::Item(id) => {
                    if let Ok(id) = get_type_id(items, scope_id, id.as_path()) {
                        return Some(id);
                    }
                }
                ComponentType::Contained {
                    type_,
                    element_type,
                } => {
                    if let Ok(id) = get_type_id(items, scope_id, element_type.as_path()) {
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

    pub(crate) fn get_attribute_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Attribute>> {
        for &scope_id in self.0.iter().rev() {
            if let Ok(id) = get_attribute_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find attribute {:?}", path);
    }

    pub(crate) fn get_concept_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Concept>> {
        for &scope_id in self.0.iter().rev() {
            if let Ok(id) = get_concept_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find concept {:?}", path);
    }

    pub(crate) fn get_component_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Component>> {
        for &scope_id in self.0.iter().rev() {
            if let Ok(id) = get_component_id(items, scope_id, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find component {:?}", path);
    }
}

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub parent: Option<ItemId<Scope>>,
    pub id: Identifier,

    pub scopes: IndexMap<Identifier, ItemId<Scope>>,
    pub components: IndexMap<Identifier, ItemId<Component>>,
    pub concepts: IndexMap<Identifier, ItemId<Concept>>,
    pub messages: IndexMap<Identifier, ItemId<Message>>,
    pub types: IndexMap<Identifier, ItemId<Type>>,
    pub attributes: IndexMap<Identifier, ItemId<Attribute>>,
}
impl std::fmt::Debug for Scope {
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
impl Item for Scope {
    const TYPE: ItemType = ItemType::Scope;

    type Unresolved = ();

    fn from_item_value(value: &ItemValue) -> Option<&Self> {
        match value {
            ItemValue::Scope(value) => Some(value),
            _ => None,
        }
    }

    fn from_item_value_mut(value: &mut ItemValue) -> Option<&mut Self> {
        match value {
            ItemValue::Scope(value) => Some(value),
            _ => None,
        }
    }

    fn into_item_value(self) -> ItemValue {
        ItemValue::Scope(self)
    }

    fn parent(&self) -> Option<ItemId<Scope>> {
        self.parent
    }

    fn id(&self) -> &Identifier {
        &self.id
    }
}
/// Scope uses `ResolveClone` because scopes can be accessed during resolution
/// of their children, so we need to clone the scope to avoid a double-borrow.
impl ResolveClone for Scope {
    fn resolve_clone(
        self,
        items: &mut ItemMap,
        self_id: ItemId<Self>,
        context: &Context,
    ) -> anyhow::Result<Self> {
        fn resolve<T: Resolve, U>(
            item_ids: &IndexMap<U, ItemId<T>>,
            items: &ItemMap,
            context: &Context,
        ) -> anyhow::Result<()> {
            for id in item_ids.values().copied() {
                items.resolve(id, context)?;
            }

            Ok(())
        }

        let mut context = context.clone();
        context.push(self_id);

        for id in self.scopes.values().copied() {
            items.resolve_clone(id, &context)?;
        }
        resolve(&self.components, items, &context)?;
        resolve(&self.concepts, items, &context)?;
        resolve(&self.messages, items, &context)?;
        resolve(&self.types, items, &context)?;
        resolve(&self.attributes, items, &context)?;

        Ok(self)
    }
}
impl Scope {
    pub fn from_manifest(
        semantic: &mut Semantic,
        parent: ItemId<Scope>,
        file_provider: &dyn FileProvider,
        manifest: Manifest,
    ) -> anyhow::Result<ItemId<Scope>> {
        let scope = Scope {
            parent: Some(parent),
            id: manifest.project.id.clone(),

            scopes: IndexMap::new(),
            components: IndexMap::new(),
            concepts: IndexMap::new(),
            messages: IndexMap::new(),
            types: IndexMap::new(),
            attributes: IndexMap::new(),
        };
        let scope_id = semantic.items.add(scope);

        for include in &manifest.project.includes {
            let child_scope_id =
                semantic.add_file_at_non_toplevel(scope_id, &include, file_provider)?;
            let id = semantic.items.get(child_scope_id)?.id.clone();
            semantic
                .items
                .get_mut(scope_id)?
                .scopes
                .insert(id, child_scope_id);
        }

        let items = &mut semantic.items;
        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Component::from_project(scope_id, item.clone(), component));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .components
                .insert(item.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_project(scope_id, item.clone(), concept));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .concepts
                .insert(item.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_project(scope_id, item.clone(), message));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .messages
                .insert(item.clone(), value);
        }

        for (segment, enum_ty) in manifest.enums.iter() {
            let enum_id = items.add(Type::from_project_enum(scope_id, segment.clone(), enum_ty));
            items
                .get_mut(scope_id)?
                .types
                .insert(segment.clone(), enum_id);
        }

        Ok(scope_id)
    }
}

fn get_type_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Type>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)?
        .types
        .get(item)
        .copied()
        .with_context(|| format!("failed to find type {item} in {scope:?}"))
}

fn get_attribute_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Attribute>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)?
        .attributes
        .get(item)
        .copied()
        .with_context(|| format!("failed to find attribute {item} in {scope:?}",))
}

fn get_concept_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Concept>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)?
        .concepts
        .get(item)
        .copied()
        .with_context(|| format!("failed to find concept {item} in {scope:?}",))
}

fn get_component_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> anyhow::Result<ItemId<Component>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)?
        .components
        .get(item)
        .copied()
        .with_context(|| format!("failed to find component {item} in {scope:?}",))
}
