use std::fmt::Debug;

use ambient_project::{Identifier, Manifest};
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context as AnyhowContext;
use convert_case::{Boundary, Case, Casing};

use indexmap::IndexMap;

mod scope;
pub use scope::{Context, Scope};

mod item;
pub use item::{Item, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId};
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
            parent: None,
            organization: None,
            id: Identifier::default(),
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

            let ty = Type::new(root_scope, id.clone(), TypeInner::Primitive(pt));
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
                parent: sem.root_scope,
                id: id.clone(),
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
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        Scope::from_manifest(self, Some(parent_scope), file_provider, manifest)
    }

    pub fn add_file(
        &mut self,
        filename: &str,
        file_provider: &dyn FileProvider,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest: Manifest = toml::from_str(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename}"))?;

        let scope_id = manifest.project.id.clone();
        if self.scopes.contains_key(&scope_id) {
            anyhow::bail!("file `{}` has already been added as {}", filename, scope_id);
        }

        if manifest.project.organization.is_none() {
            anyhow::bail!("file `{}` has no organization, which is required", filename);
        }

        let item_id = Scope::from_manifest(self, None, file_provider, manifest)?;
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
