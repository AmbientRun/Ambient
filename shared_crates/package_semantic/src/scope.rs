use ambient_package::{PascalCaseIdentifier, SnakeCaseIdentifier};
use indexmap::IndexMap;

use crate::{
    Attribute, Component, Concept, Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, Message,
    Package, Resolve, Semantic, Type,
};

#[derive(Clone, PartialEq)]
pub struct Scope {
    pub data: ItemData,

    pub imports: IndexMap<SnakeCaseIdentifier, ItemId<Package>>,
    pub scopes: IndexMap<SnakeCaseIdentifier, ItemId<Scope>>,
    pub components: IndexMap<SnakeCaseIdentifier, ItemId<Component>>,
    pub concepts: IndexMap<PascalCaseIdentifier, ItemId<Concept>>,
    pub messages: IndexMap<PascalCaseIdentifier, ItemId<Message>>,
    pub types: IndexMap<PascalCaseIdentifier, ItemId<Type>>,
    pub attributes: IndexMap<PascalCaseIdentifier, ItemId<Attribute>>,

    resolved: bool,
}
impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_struct("Scope");
        ds.field("data", &self.data);

        if !self.scopes.is_empty() {
            ds.field("scopes", &self.scopes);
        }
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

    fn data(&self) -> &ItemData {
        &self.data
    }
}
/// Scope uses `ResolveClone` because scopes can be accessed during resolution
/// of their children, so we need to clone the scope to avoid a double-borrow.
impl Resolve for Scope {
    fn resolve(mut self, semantic: &mut Semantic, _self_id: ItemId<Self>) -> anyhow::Result<Self> {
        fn resolve<T: Resolve, U>(
            semantic: &mut Semantic,
            item_ids: &IndexMap<U, ItemId<T>>,
        ) -> anyhow::Result<()> {
            for id in item_ids.values().copied() {
                semantic.resolve(id)?;
            }

            Ok(())
        }

        resolve(semantic, &self.scopes)?;
        resolve(semantic, &self.components)?;
        resolve(semantic, &self.concepts)?;
        resolve(semantic, &self.messages)?;
        resolve(semantic, &self.types)?;
        resolve(semantic, &self.attributes)?;

        self.resolved = true;

        Ok(self)
    }

    fn already_resolved(&self) -> bool {
        self.resolved
    }
}
impl Scope {
    /// Creates a new empty scope with the specified data.
    pub fn new(data: ItemData) -> Self {
        Self {
            data,
            imports: Default::default(),
            scopes: Default::default(),
            components: Default::default(),
            concepts: Default::default(),
            messages: Default::default(),
            types: Default::default(),
            attributes: Default::default(),
            resolved: false,
        }
    }

    pub fn visit_recursive(
        &self,
        items: &ItemMap,
        mut visitor: impl FnMut(&Scope) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        fn visit_recursive_inner(
            scope: &Scope,
            items: &ItemMap,
            visitor: &mut dyn FnMut(&Scope) -> anyhow::Result<()>,
        ) -> anyhow::Result<()> {
            visitor(scope)?;

            for scope in scope.scopes.values().copied() {
                visit_recursive_inner(&items.get(scope), items, visitor)?;
            }

            Ok(())
        }

        visit_recursive_inner(self, items, &mut visitor)
    }
}
