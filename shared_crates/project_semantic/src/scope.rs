use ambient_project::{ComponentType, Identifier, ItemPath, Manifest};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Attribute, Component, Concept, FileProvider, Item, ItemId, ItemMap, ItemType, ItemValue,
    Message, Type,
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
        items: &mut ItemMap,
        component_type: &ComponentType,
    ) -> Option<ItemId<Type>> {
        for &scope_id in self.0.iter().rev() {
            match component_type {
                ComponentType::Item(id) => {
                    let scope = items.get_without_resolve(scope_id).ok()?;
                    if let Ok(id) = scope.get_type_id(items, id.as_path()) {
                        return Some(id);
                    }
                }
                ComponentType::Contained {
                    type_,
                    element_type,
                } => {
                    let scope = items.get_without_resolve(scope_id).ok()?;
                    if let Ok(id) = scope.get_type_id(items, element_type.as_path()) {
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
            let Ok(scope) = items.get_without_resolve(scope_id) else { continue; };
            if let Ok(id) = scope.get_attribute_id(items, path) {
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
            let Ok(scope) = items.get_without_resolve(scope_id) else { continue; };
            if let Ok(id) = scope.get_concept_id(items, path) {
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
            let Ok(scope) = items.get_without_resolve(scope_id) else { continue; };
            if let Ok(id) = scope.get_component_id(items, path) {
                return Ok(id);
            }
        }
        anyhow::bail!("failed to find component {:?}", path);
    }
}

#[derive(Clone, PartialEq)]
pub struct Scope {
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

    fn resolve(
        self,
        items: &mut ItemMap,
        self_id: ItemId<Self>,
        context: &Context,
    ) -> anyhow::Result<Self> {
        fn resolve<T: Item, U>(
            item_ids: &IndexMap<U, ItemId<T>>,
            items: &mut ItemMap,
            context: &Context,
        ) -> anyhow::Result<()> {
            for id in item_ids.values().copied() {
                items.resolve(id, context)?;
            }

            Ok(())
        }

        let mut context = context.clone();
        context.push(self_id);

        resolve(&self.scopes, items, &context)?;
        resolve(&self.components, items, &context)?;
        resolve(&self.concepts, items, &context)?;
        resolve(&self.messages, items, &context)?;
        resolve(&self.types, items, &context)?;
        resolve(&self.attributes, items, &context)?;

        Ok(self)
    }
}
impl Scope {
    pub fn from_file(
        items: &mut ItemMap,
        filename: &str,
        file_provider: &dyn FileProvider,
    ) -> anyhow::Result<Self> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let mut scopes = IndexMap::new();
        for include in &manifest.project.includes {
            let scope = Scope::from_file(items, include, file_provider)?;
            scopes.insert(scope.id.clone(), items.add(scope));
        }

        let mut scope = Scope {
            id: manifest.project.id.clone(),
            scopes,

            components: IndexMap::new(),
            concepts: IndexMap::new(),
            messages: IndexMap::new(),
            types: IndexMap::new(),
            attributes: IndexMap::new(),
        };

        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Component::from_project(item.clone(), component));
            scope
                .get_or_create_scope_mut(items, scope_path)
                .components
                .insert(item.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_project(item.clone(), concept));
            scope
                .get_or_create_scope_mut(items, scope_path)
                .concepts
                .insert(item.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_project(item.clone(), message));
            scope
                .get_or_create_scope_mut(items, scope_path)
                .messages
                .insert(item.clone(), value);
        }

        for (segment, ty) in manifest.enums.iter() {
            scope.types.insert(
                segment.clone(),
                items.add(Type::from_project_enum(segment.clone(), ty)),
            );
        }

        Ok(scope)
    }

    fn get_scope<'a>(
        &'a self,
        items: &'a ItemMap,
        path: &[Identifier],
    ) -> anyhow::Result<&'a Scope> {
        let mut scope = self;
        for segment in path.iter() {
            scope =
                items.get_without_resolve(*scope.scopes.get(segment).with_context(|| {
                    format!(
                        "failed to find scope {segment} in {scope}",
                        segment = segment,
                        scope = scope.id
                    )
                })?)?;
        }
        Ok(scope)
    }

    fn get_or_create_scope_mut<'a>(
        &'a mut self,
        items: &'a mut ItemMap,
        path: &[Identifier],
    ) -> &'a mut Scope {
        let mut scope_id = None;
        for segment in path.iter() {
            let scope = match scope_id {
                Some(id) => items.get_mut(id).unwrap(),
                None => self,
            };
            let new_scope_id = match scope.scopes.get(segment) {
                Some(id) => *id,
                None => items.add(Scope {
                    id: segment.clone(),
                    scopes: Default::default(),
                    components: Default::default(),
                    concepts: Default::default(),
                    messages: Default::default(),
                    types: Default::default(),
                    attributes: Default::default(),
                }),
            };
            scope_id = Some(new_scope_id);
        }
        match scope_id {
            Some(id) => items.get_mut(id).unwrap(),
            None => self,
        }
    }

    fn get_type_id(&self, items: &ItemMap, path: ItemPath) -> anyhow::Result<ItemId<Type>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(items, scope)?
            .types
            .get(item)
            .copied()
            .with_context(|| format!("failed to find type {item} in {scope:?}",))
    }

    fn get_attribute_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Attribute>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(items, scope)?
            .attributes
            .get(item)
            .copied()
            .with_context(|| format!("failed to find attribute {item} in {scope:?}",))
    }

    fn get_concept_id(&self, items: &ItemMap, path: ItemPath) -> anyhow::Result<ItemId<Concept>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(items, scope)?
            .concepts
            .get(item)
            .copied()
            .with_context(|| format!("failed to find concept {item} in {scope:?}",))
    }

    fn get_component_id(
        &self,
        items: &ItemMap,
        path: ItemPath,
    ) -> anyhow::Result<ItemId<Component>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(items, scope)?
            .components
            .get(item)
            .copied()
            .with_context(|| format!("failed to find component {item} in {scope:?}",))
    }
}
