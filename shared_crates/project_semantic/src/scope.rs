use ambient_project::{ComponentType, Identifier, ItemPath, Manifest};
use anyhow::Context as AnyhowContext;
use indexmap::IndexMap;

use crate::{
    Attribute, Component, Concept, FileProvider, Item, ItemId, ItemMap, Message, Semantic, Type,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Context<'a>(Vec<&'a Scope>);
impl<'a> Context<'a> {
    pub(crate) fn new(root_scope: &'a Scope) -> Self {
        Self(vec![root_scope])
    }

    fn push(&mut self, scope: &'a Scope) {
        self.0.push(scope);
    }

    pub(crate) fn get_type_id(
        &self,
        items: &mut ItemMap,
        component_type: &ComponentType,
    ) -> Option<ItemId<Type>> {
        for scope in self.0.iter().rev() {
            match component_type {
                ComponentType::Item(id) => {
                    if let Some(id) = scope.get_type_id(id.as_path()) {
                        return Some(id);
                    }
                }
                ComponentType::Contained {
                    type_,
                    element_type,
                } => {
                    if let Some(id) = scope.get_type_id(element_type.as_path()) {
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

    pub(crate) fn get_attribute_id(&self, path: ItemPath) -> Option<ItemId<Attribute>> {
        for scope in self.0.iter().rev() {
            if let Some(id) = scope.get_attribute_id(path) {
                return Some(id);
            }
        }
        None
    }

    pub(crate) fn get_concept_id(&self, path: ItemPath) -> Option<ItemId<Concept>> {
        for scope in self.0.iter().rev() {
            if let Some(id) = scope.get_concept_id(path) {
                return Some(id);
            }
        }
        None
    }

    pub(crate) fn get_component_id(&self, path: ItemPath) -> Option<ItemId<Component>> {
        for scope in self.0.iter().rev() {
            if let Some(id) = scope.get_component_id(path) {
                return Some(id);
            }
        }
        None
    }
}

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub id: Identifier,
    pub scopes: IndexMap<Identifier, Scope>,

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
impl Scope {
    pub fn from_file(
        semantic: &mut Semantic,
        filename: &str,
        file_provider: &dyn FileProvider,
    ) -> anyhow::Result<Self> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let mut scopes = IndexMap::new();
        for include in &manifest.project.includes {
            let scope = Scope::from_file(semantic, include, file_provider)?;
            scopes.insert(scope.id.clone(), scope);
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

            scope.get_or_create_scope_mut(scope_path).components.insert(
                item.clone(),
                semantic
                    .items
                    .add(Component::from_project(item.clone(), component)),
            );
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            scope.get_or_create_scope_mut(scope_path).concepts.insert(
                item.clone(),
                semantic
                    .items
                    .add(Concept::from_project(item.clone(), concept)),
            );
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            scope.get_or_create_scope_mut(scope_path).messages.insert(
                item.clone(),
                semantic
                    .items
                    .add(Message::from_project(item.clone(), message)),
            );
        }

        for (segment, ty) in manifest.enums.iter() {
            scope.types.insert(
                segment.clone(),
                semantic
                    .items
                    .add(Type::from_project_enum(segment.clone(), ty)),
            );
        }

        Ok(scope)
    }

    fn get_scope(&self, path: &[Identifier]) -> Option<&Scope> {
        let mut scope = self;
        for segment in path.iter() {
            scope = scope.scopes.get(segment)?;
        }
        Some(scope)
    }

    fn get_or_create_scope_mut(&mut self, path: &[Identifier]) -> &mut Scope {
        let mut scope = self;
        for segment in path.iter() {
            scope = scope
                .scopes
                .entry(segment.clone())
                .or_insert_with(|| Scope {
                    id: segment.clone(),
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

    pub(crate) fn resolve<'a>(&'a self, items: &mut ItemMap, mut context: Context<'a>) {
        fn resolve<T: Item, U>(
            item_ids: &IndexMap<U, ItemId<T>>,
            items: &mut ItemMap,
            context: &Context,
        ) {
            for id in item_ids.values().copied() {
                items.resolve(id, context);
            }
        }

        context.push(self);

        resolve(&self.components, items, &context);
        resolve(&self.concepts, items, &context);
        resolve(&self.messages, items, &context);
        resolve(&self.types, items, &context);
        resolve(&self.attributes, items, &context);

        for scope in self.scopes.values() {
            scope.resolve(items, context.clone());
        }
    }

    fn get_type_id(&self, path: ItemPath) -> Option<ItemId<Type>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(scope)?.types.get(item).copied()
    }

    fn get_attribute_id(&self, path: ItemPath) -> Option<ItemId<Attribute>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(scope)?.attributes.get(item).copied()
    }

    fn get_concept_id(&self, path: ItemPath) -> Option<ItemId<Concept>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(scope)?.concepts.get(item).copied()
    }

    fn get_component_id(&self, path: ItemPath) -> Option<ItemId<Component>> {
        let (scope, item) = path.scope_and_item();
        self.get_scope(scope)?.components.get(item).copied()
    }
}
