use std::{
    cell::Ref,
    fmt::Debug,
    path::{Path, PathBuf},
};

use ambient_project::{Dependency, Identifier, Manifest};
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context as AnyhowContext;
use convert_case::{Boundary, Case, Casing};

mod scope;
pub use scope::{Context, Scope};

mod item;
pub use item::{
    Item, ItemData, ItemId, ItemMap, ItemSource, ItemType, ItemValue, ResolvableItemId,
};
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
pub use value::{ResolvableValue, ScalarValue, Value};

mod printer;
pub use printer::Printer;

pub trait FileProvider {
    fn get(&self, path: &Path) -> std::io::Result<String>;
    fn full_path(&self, path: &Path) -> PathBuf;
}

/// Implements [FileProvider] by reading from the filesystem.
pub struct DiskFileProvider(pub PathBuf);
impl FileProvider for DiskFileProvider {
    fn get(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(self.0.join(path))
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        self.0.join(path)
    }
}

/// Implements [FileProvider] by reading from an array of files.
///
/// Used with `ambient_schema`.
pub struct ArrayFileProvider<'a> {
    pub files: &'a [(&'a str, &'a str)],
}
impl ArrayFileProvider<'_> {
    pub fn from_schema() -> Self {
        Self {
            files: ambient_schema::FILES,
        }
    }
}
impl FileProvider for ArrayFileProvider<'_> {
    fn get(&self, path: &Path) -> std::io::Result<String> {
        let path = path.to_str().unwrap();
        for (name, contents) in self.files {
            if path == *name {
                return Ok(contents.to_string());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("file not found: {:?}", path),
        ))
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }
}

pub struct ProxyFileProvider<'a> {
    pub provider: &'a dyn FileProvider,
    pub base: &'a Path,
}
impl FileProvider for ProxyFileProvider<'_> {
    fn get(&self, path: &Path) -> std::io::Result<String> {
        self.provider.get(&self.base.join(path))
    }

    fn full_path(&self, path: &Path) -> PathBuf {
        ambient_shared_types::path::normalize(&self.provider.full_path(&self.base.join(path)))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    pub items: ItemMap,
    pub root_scope_id: ItemId<Scope>,
}
impl Semantic {
    pub fn new() -> anyhow::Result<Self> {
        let mut items = ItemMap::default();
        let root_scope_id = create_root_scope(&mut items)?;
        Ok(Self {
            items,
            root_scope_id,
        })
    }

    pub fn add_file_at_non_toplevel(
        &mut self,
        parent_scope: ItemId<Scope>,
        filename: &Path,
        file_provider: &dyn FileProvider,
        source: ItemSource,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest = Manifest::parse(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename:?}"))?;

        let id = manifest.ember.id.clone();
        self.add_scope_from_manifest(
            Some(parent_scope),
            file_provider,
            manifest,
            file_provider.full_path(filename),
            id,
            source,
        )
    }

    // TODO(philpax): This merges scopes together, which may lead to some degree of semantic conflation,
    // especially with dependencies: a parent may be able to access a child's dependencies.
    //
    // This is a simplifying assumption that will enable the cross-cutting required for Ambient's ecosystem,
    // but will lead to unexpected behaviour in future.
    //
    // A fix may be to treat each added manifest as an "island", and then have the resolution step
    // jump between islands as required to resolve things. There are a couple of nuances here that
    // I decided to push to another day in the interest of getting this working.
    //
    // These nuances include:
    // - Sharing the same "ambient" types between islands (primitive types, Ambient API)
    // - If one module/island (P) has dependencies on two islands (A, B), both of which have a shared dependency (C),
    //   both A and B should have the same C and not recreate it. C should not be visible from P.
    // - Local changes should not have global effects, unless they are globally visible. If, using the above configuration,
    //   a change occurs to C, there should be absolutely no impact on P if P does not depend on C.
    //
    // At the present, there's just one big island, so P can see C, and changes to C will affect P.
    pub fn add_file(
        &mut self,
        filename: &Path,
        file_provider: &dyn FileProvider,
        source: ItemSource,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest = Manifest::parse(&file_provider.get(filename)?)
            .with_context(|| format!("failed to parse toml for {filename:?}"))?;

        let root_id = self.root_scope_id;

        // Check that this scope hasn't already been created for this scope
        let scope_id = manifest.ember.id.clone();
        if let Some(existing_scope_id) = self.items.get(root_id)?.scopes.get(&scope_id) {
            let existing_path = self.items.get(*existing_scope_id)?.path.clone();
            if existing_path == Some(file_provider.full_path(filename)) {
                return Ok(*existing_scope_id);
            }

            anyhow::bail!(
                "attempted to add {:?}, but a scope already exists at `{scope_id}`",
                file_provider.full_path(filename)
            );
        }

        // Create a new scope and add it to the scope
        let manifest_path = file_provider.full_path(filename);
        let item_id = self.add_scope_from_manifest(
            Some(root_id),
            file_provider,
            manifest,
            manifest_path,
            scope_id.clone(),
            source,
        )?;
        self.items
            .get_mut(root_id)?
            .scopes
            .insert(scope_id, item_id);
        Ok(item_id)
    }

    pub fn resolve(&mut self) -> anyhow::Result<()> {
        let root_scopes = self
            .items
            .get(self.root_scope_id)?
            .scopes
            .values()
            .copied()
            .collect::<Vec<_>>();

        for scope_id in root_scopes {
            self.items
                .resolve_clone(scope_id, &Context::new(self.root_scope_id))?;
        }
        Ok(())
    }

    pub fn root_scope(&self) -> Ref<'_, Scope> {
        self.items.get(self.root_scope_id).unwrap()
    }
}
impl Semantic {
    #[allow(clippy::too_many_arguments)]
    fn add_scope_from_manifest(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        file_provider: &dyn FileProvider,
        manifest: Manifest,
        manifest_path: PathBuf,
        id: Identifier,
        source: ItemSource,
    ) -> anyhow::Result<ItemId<Scope>> {
        let scope = Scope::new(
            ItemData {
                parent_id,
                id,
                source,
            },
            Some(manifest_path.clone()),
            Some(manifest.clone()),
        );
        let scope_id = self.items.add(scope);

        for include in &manifest.ember.includes {
            let child_scope_id =
                self.add_file_at_non_toplevel(scope_id, include, file_provider, source)?;
            let id = self.items.get(child_scope_id)?.data().id.clone();
            self.items
                .get_mut(scope_id)?
                .scopes
                .insert(id, child_scope_id);
        }

        for (_, dependency) in manifest.dependencies.iter() {
            match dependency {
                Dependency::Path { path } => {
                    let file_provider = ProxyFileProvider {
                        provider: file_provider,
                        base: path,
                    };

                    self.add_file(Path::new("ambient.toml"), &file_provider, source)?;
                }
            }
        }

        let make_item_data = |item_id: &Identifier| -> ItemData {
            ItemData {
                parent_id: Some(scope_id),
                id: item_id.clone(),
                source,
            }
        };

        let items = &mut self.items;
        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Component::from_project(make_item_data(item), component));
            items
                .get_or_create_scope_mut(manifest_path.clone(), scope_id, scope_path)?
                .components
                .insert(item.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_project(make_item_data(item), concept));
            items
                .get_or_create_scope_mut(manifest_path.clone(), scope_id, scope_path)?
                .concepts
                .insert(item.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_project(make_item_data(item), message));
            items
                .get_or_create_scope_mut(manifest_path.clone(), scope_id, scope_path)?
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
// public helpers
impl Semantic {
    pub fn add_ambient_schema(&mut self) -> anyhow::Result<ItemId<Scope>> {
        self.add_file(
            Path::new("ambient.toml"),
            &ArrayFileProvider::from_schema(),
            ItemSource::Ambient,
        )
    }

    pub fn add_ember(&mut self, ember_path: &Path) -> anyhow::Result<ItemId<Scope>> {
        self.add_file(
            Path::new("ambient.toml"),
            &DiskFileProvider(ember_path.to_owned()),
            ItemSource::User,
        )
    }
}

fn create_root_scope(items: &mut ItemMap) -> anyhow::Result<ItemId<Scope>> {
    macro_rules! define_primitive_types {
        ($(($value:ident, $_type:ty)),*) => {
            [
                $((stringify!($value), PrimitiveType::$value)),*
            ]
        };
    }

    let root_scope = items.add(Scope::new(
        ItemData {
            parent_id: None,
            id: Identifier::default(),
            source: ItemSource::System,
        },
        None,
        None,
    ));

    for (id, pt) in primitive_component_definitions!(define_primitive_types) {
        let id = id
            .with_boundaries(&[
                Boundary::LowerUpper,
                Boundary::DigitUpper,
                Boundary::DigitLower,
                Boundary::Acronym,
            ])
            .to_case(Case::Snake);
        let id = Identifier::new(id)
            .map_err(anyhow::Error::msg)
            .context("standard value was not valid snake-case")?;

        let ty = Type::new(
            ItemData {
                parent_id: Some(root_scope),
                id: id.clone(),
                source: ItemSource::System,
            },
            TypeInner::Primitive(pt),
        );
        let item_id = items.add(ty);
        items.get_mut(root_scope)?.types.insert(id, item_id);
    }

    for name in [
        "debuggable",
        "networked",
        "resource",
        "maybe_resource",
        "store",
    ] {
        let id = Identifier::new(name)
            .map_err(anyhow::Error::msg)
            .context("standard value was not valid snake-case")?;
        let item_id = items.add(Attribute {
            data: ItemData {
                parent_id: Some(root_scope),
                id: id.clone(),
                source: ItemSource::System,
            },
        });
        items.get_mut(root_scope)?.attributes.insert(id, item_id);
    }

    Ok(root_scope)
}
