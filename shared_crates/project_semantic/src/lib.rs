use std::fmt::Debug;

use ambient_project::Identifier;
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context as AnyhowContext;
use convert_case::{Boundary, Case, Casing};

use indexmap::IndexMap;

mod scope;
pub use scope::{Context, Scope};

mod item;
pub use item::{Item, ItemId, ItemMap, ItemType, ItemValue, ResolvableItemId};

mod component;
pub use component::Component;

mod concept;
pub use concept::Concept;

mod attribute;
pub use attribute::Attribute;

mod primitive_type;
pub use primitive_type::PrimitiveType;

mod type_;
pub use type_::{Enum, Type};

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
    root_scope: Scope,
    pub scopes: IndexMap<Identifier, Scope>,
}
impl Semantic {
    pub fn new() -> anyhow::Result<Self> {
        macro_rules! define_primitive_types {
            ($(($value:ident, $_type:ty)),*) => {
                [
                    $((stringify!($value), Type::Primitive(PrimitiveType::$value))),*
                ]
            };
        }

        let mut sem = Self {
            items: ItemMap::default(),
            root_scope: Scope {
                id: Identifier::default(),
                scopes: IndexMap::new(),
                components: IndexMap::new(),
                concepts: IndexMap::new(),
                messages: IndexMap::new(),
                types: IndexMap::new(),
                attributes: IndexMap::new(),
            },
            scopes: IndexMap::new(),
        };

        for (id, ty) in primitive_component_definitions!(define_primitive_types) {
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

            let item_id = sem.items.add(ty);
            sem.root_scope.types.insert(id, item_id);
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
            let item_id = sem.items.add(Attribute { id: id.clone() });
            sem.root_scope.attributes.insert(id, item_id);
        }

        Ok(sem)
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
            scope.resolve(&mut self.items, Context::new(&self.root_scope))?;
        }
        Ok(())
    }
}
