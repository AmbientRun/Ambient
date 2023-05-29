use std::fmt::Debug;

use ambient_project::{Identifier, Manifest};
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context as AnyhowContext;
use convert_case::{Boundary, Case, Casing};

use indexmap::IndexMap;

mod scope;
pub use scope::{Context, Scope};

mod item;
pub use item::{Item, ItemData, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId};
use item::{Resolve, ResolveClone};

mod component;
pub use component::Component;

mod concept;
pub use concept::Concept;

mod attribute;
pub use attribute::Attribute;

mod primitive_type;
pub use primitive_type::PrimitiveType;

mod type_;
pub use type_::{Enum, Type, TypeInner};

mod message;
pub use message::Message;

mod value;
pub use value::{PrimitiveValue, ResolvableValue, ResolvedValue};

pub trait FileProvider {
    fn get(&self, filename: &str) -> std::io::Result<String>;
}

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    pub items: ItemMap,
    pub root_scope: ItemId<Scope>,
    pub scopes: IndexMap<Identifier, ItemId<Scope>>,
}
impl Semantic {
    pub fn new() -> anyhow::Result<Self> {
        macro_rules! define_primitive_types {
            ($(($value:ident, $_type:ty)),*) => {
                [
                    $((stringify!($value), PrimitiveType::$value)),*
                ]
            };
        }

        let mut items = ItemMap::default();
        let root_scope = items.add(Scope {
            data: ItemData {
                parent_id: None,
                id: Identifier::default(),
                is_ambient: true,
            },
            organization: None,
            scopes: IndexMap::new(),
            components: IndexMap::new(),
            concepts: IndexMap::new(),
            messages: IndexMap::new(),
            types: IndexMap::new(),
            attributes: IndexMap::new(),
        });
        let mut sem = Self {
            items,
            root_scope,
            scopes: IndexMap::new(),
        };

        for (id, pt) in primitive_component_definitions!(define_primitive_types) {
            let id = id
                .with_boundaries(&[
                    Boundary::LowerUpper,
                    Boundary::DigitUpper,
                    Boundary::DigitLower,
                    Boundary::Acronym,
                ])
                .to_case(Case::Kebab);
            let id = Identifier::new(id)
                .map_err(anyhow::Error::msg)
                .context("standard value was not valid kebab-case")?;

            let ty = Type::new(
                ItemData {
                    parent_id: Some(root_scope),
                    id: id.clone(),
                    is_ambient: true,
                },
                TypeInner::Primitive(pt),
            );
            let item_id = sem.items.add(ty);
            sem.items.get_mut(sem.root_scope)?.types.insert(id, item_id);
        }

        for name in [
            "debuggable",
            "networked",
            "resource",
            "maybe-resource",
            "store",
        ] {
            let id = Identifier::new(name)
                .map_err(anyhow::Error::msg)
                .context("standard value was not valid kebab-case")?;
            let item_id = sem.items.add(Attribute {
                data: ItemData {
                    parent_id: Some(sem.root_scope),
                    id: id.clone(),
                    is_ambient: true,
                },
            });
            sem.items
                .get_mut(sem.root_scope)?
                .attributes
                .insert(id, item_id);
        }

        Ok(sem)
    }

    pub fn add_file_at_non_toplevel(
        &mut self,
        parent_scope: ItemId<Scope>,
        filename: &str,
        file_provider: &dyn FileProvider,
        is_ambient: bool,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        self.add_scope_from_manifest(Some(parent_scope), file_provider, manifest, is_ambient)
    }

    pub fn add_file(
        &mut self,
        filename: &str,
        file_provider: &dyn FileProvider,
        is_ambient: bool,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let scope_id = manifest.project.id.clone();
        if self.scopes.contains_key(&scope_id) {
            anyhow::bail!(
                "attempted to add {filename}, but a scope already exists at `{scope_id}`"
            );
        }

        if manifest.project.organization.is_none() {
            anyhow::bail!("file `{}` has no organization, which is required", filename);
        }

        let item_id = self.add_scope_from_manifest(None, file_provider, manifest, is_ambient)?;
        self.scopes.insert(scope_id, item_id);
        Ok(item_id)
    }

    pub fn resolve(&mut self) -> anyhow::Result<()> {
        for &scope_id in self.scopes.values() {
            self.items
                .resolve_clone(scope_id, &Context::new(self.root_scope))?;
        }
        Ok(())
    }
}
impl Semantic {
    fn add_scope_from_manifest(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        file_provider: &dyn FileProvider,
        manifest: Manifest,
        is_ambient: bool,
    ) -> anyhow::Result<ItemId<Scope>> {
        let scope = Scope {
            data: ItemData {
                parent_id,
                id: manifest.project.id.clone(),
                is_ambient,
            },
            organization: manifest.project.organization.clone(),

            scopes: IndexMap::new(),
            components: IndexMap::new(),
            concepts: IndexMap::new(),
            messages: IndexMap::new(),
            types: IndexMap::new(),
            attributes: IndexMap::new(),
        };
        let scope_id = self.items.add(scope);

        for include in &manifest.project.includes {
            let child_scope_id =
                self.add_file_at_non_toplevel(scope_id, include, file_provider, is_ambient)?;
            let id = self.items.get(child_scope_id)?.data().id.clone();
            self.items
                .get_mut(scope_id)?
                .scopes
                .insert(id, child_scope_id);
        }

        let make_item_data = |item_id: &Identifier| -> ItemData {
            ItemData {
                parent_id: Some(scope_id),
                id: item_id.clone(),
                is_ambient,
            }
        };

        let items = &mut self.items;
        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Component::from_project(make_item_data(item), component));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .components
                .insert(item.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_project(make_item_data(item), concept));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .concepts
                .insert(item.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_project(make_item_data(item), message));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .messages
                .insert(item.clone(), value);
        }

        for (segment, enum_ty) in manifest.enums.iter() {
            let enum_id = items.add(Type::from_project_enum(make_item_data(segment), enum_ty));
            items
                .get_mut(scope_id)?
                .types
                .insert(segment.clone(), enum_id);
        }

        Ok(scope_id)
    }
}
