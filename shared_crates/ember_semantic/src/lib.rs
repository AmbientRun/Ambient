use std::{
    cell::Ref,
    collections::HashSet,
    fmt::Debug,
    path::{Path, PathBuf},
};

use ambient_ember::{
    BuildMetadata, Dependency, Identifier, Manifest, PascalCaseIdentifier, SnakeCaseIdentifier,
};
use ambient_shared_types::primitive_component_definitions;
use anyhow::Context as AnyhowContext;

mod scope;
use async_recursion::async_recursion;
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

mod providers;
pub use providers::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    pub items: ItemMap,
    pub root_scope_id: ItemId<Scope>,
    pub standard_definitions: StandardDefinitions,
}
impl Semantic {
    pub async fn new() -> anyhow::Result<Self> {
        let mut items = ItemMap::default();
        let (root_scope_id, standard_definitions) = create_root_scope(&mut items)?;
        let mut semantic = Self {
            items,
            root_scope_id,
            standard_definitions,
        };

        semantic
            .add_file(
                Path::new("ambient.toml"),
                &ArrayFileProvider::from_schema(),
                ItemSource::Ambient,
                None,
            )
            .await?;

        Ok(semantic)
    }

    pub async fn add_file(
        &mut self,
        filename: &Path,
        file_provider: &dyn FileProvider,
        source: ItemSource,
        scope_name: Option<SnakeCaseIdentifier>,
    ) -> anyhow::Result<ItemId<Scope>> {
        self.add_file_internal(
            filename,
            file_provider,
            &mut HashSet::new(),
            source,
            scope_name,
            true,
        )
        .await
    }

    pub async fn add_ember(&mut self, ember_path: &Path) -> anyhow::Result<ItemId<Scope>> {
        self.add_file(
            Path::new("ambient.toml"),
            &DiskFileProvider(ember_path.to_owned()),
            ItemSource::User,
            None,
        )
        .await
    }

    /// HACK! This is used to allow the use of no-dependency embers for now.
    /// Will be removed once we figure this out.
    pub async fn add_ember_manifest(
        &mut self,
        ember_manifest: &Manifest,
    ) -> anyhow::Result<ItemId<Scope>> {
        self.add_file(
            Path::new("ambient.toml"),
            &ArrayFileProvider {
                files: &[("ambient.toml", &toml::to_string(ember_manifest)?)],
            },
            ItemSource::User,
            None,
        )
        .await
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
            self.items.resolve_clone(
                &Context::new(self.root_scope_id),
                &self.standard_definitions,
                scope_id,
            )?;
        }
        Ok(())
    }

    pub fn root_scope(&self) -> Ref<'_, Scope> {
        self.items.get(self.root_scope_id).unwrap()
    }

    pub fn get_scope_id_by_name(&self, name: &SnakeCaseIdentifier) -> Option<ItemId<Scope>> {
        self.root_scope().scopes.get(name).copied()
    }
}
impl Semantic {
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
    #[async_recursion]
    async fn add_file_internal(
        &mut self,
        filename: &Path,
        file_provider: &dyn FileProvider,
        visited_files: &mut HashSet<PathBuf>,
        source: ItemSource,
        scope_name: Option<SnakeCaseIdentifier>,
        enabled: bool,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest = Manifest::parse(&file_provider.get(filename).await.with_context(|| {
            format!(
                "failed to read top-level file {:?}",
                file_provider.full_path(filename)
            )
        })?)
        .with_context(|| {
            format!(
                "failed to parse toml for {:?}",
                file_provider.full_path(filename)
            )
        })?;

        let build_metadata_path = Path::new(BuildMetadata::FILENAME);
        let build_metadata = file_provider
            .get(build_metadata_path)
            .await
            .ok()
            .map(|f| BuildMetadata::parse(&f))
            .transpose()
            .with_context(|| {
                format!(
                    "failed to parse build metadata for {:?}",
                    file_provider.full_path(build_metadata_path)
                )
            })?;

        let root_id = self.root_scope_id;

        // Check that this scope hasn't already been created for this scope
        let scope_name = scope_name.unwrap_or_else(|| manifest.ember.id.clone());
        if let Some(existing_scope_id) = self.items.get(root_id)?.scopes.get(&scope_name) {
            let existing_path = self.items.get(*existing_scope_id)?.manifest_path.clone();
            if existing_path == Some(file_provider.full_path(filename)) {
                return Ok(*existing_scope_id);
            }

            anyhow::bail!(
                "attempted to add {:?}, but a scope already exists at `{scope_name}`",
                file_provider.full_path(filename)
            );
        }

        // Create a new scope and add it to the root scope
        let manifest_path = file_provider.full_path(filename);
        let item_id = self
            .add_scope_from_manifest(
                Some(root_id),
                file_provider,
                visited_files,
                manifest,
                manifest_path,
                build_metadata,
                scope_name.clone(),
                source,
                enabled,
            )
            .await?;
        self.items
            .get_mut(root_id)?
            .scopes
            .insert(scope_name, item_id);
        Ok(item_id)
    }

    #[async_recursion]
    async fn add_file_at_non_toplevel(
        &mut self,
        parent_scope: ItemId<Scope>,
        filename: &Path,
        file_provider: &dyn FileProvider,
        visited_files: &mut HashSet<PathBuf>,
        source: ItemSource,
        enabled: bool,
    ) -> anyhow::Result<ItemId<Scope>> {
        let manifest = Manifest::parse(&file_provider.get(filename).await.with_context(|| {
            format!("failed to read file {filename:?} within parent scope {parent_scope:?}")
        })?)
        .with_context(|| format!("failed to parse toml for {filename:?}"))?;

        let id = manifest.ember.id.clone();
        self.add_scope_from_manifest(
            Some(parent_scope),
            file_provider,
            visited_files,
            manifest,
            file_provider.full_path(filename),
            None,
            id,
            source,
            enabled,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_scope_from_manifest(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        file_provider: &dyn FileProvider,
        visited_files: &mut HashSet<PathBuf>,
        manifest: Manifest,
        manifest_path: PathBuf,
        build_metadata: Option<BuildMetadata>,
        id: SnakeCaseIdentifier,
        source: ItemSource,
        enabled: bool,
    ) -> anyhow::Result<ItemId<Scope>> {
        let mut scope = Scope::new(
            ItemData {
                parent_id,
                id: id.into(),
                source,
            },
            manifest.ember.id.clone(),
            Some(manifest_path.clone()),
            Some(manifest.clone()),
            enabled,
        );
        scope.build_metadata = build_metadata;
        let scope_id = self.items.add(scope);

        let full_path = file_provider.full_path(&manifest_path);
        if !visited_files.insert(full_path.clone()) {
            anyhow::bail!("circular dependency detected at {manifest_path:?}; previously visited files: {visited_files:?}");
        }

        for include in &manifest.ember.includes {
            let child_scope_id = self
                .add_file_at_non_toplevel(
                    scope_id,
                    include,
                    file_provider,
                    visited_files,
                    source,
                    enabled,
                )
                .await?;
            let id = self.items.get(child_scope_id)?.data().id.clone();
            self.items
                .get_mut(scope_id)?
                .scopes
                .insert(id.as_snake()?.clone(), child_scope_id);
        }

        let mut dependency_scopes = vec![];
        for (dependency_name, dependency) in manifest.dependencies.iter() {
            let Dependency { path, enabled } = dependency;

            let file_provider = ProxyFileProvider {
                provider: file_provider,
                base: path,
            };

            let ambient_toml = Path::new("ambient.toml");
            let new_scope_id = self
                        .add_file_internal(
                            ambient_toml,
                            &file_provider,
                            visited_files,
                            source,
                            Some(dependency_name.clone()),
                            *enabled,
                        ).await
                        .with_context(|| {
                            format!(
                            "failed to add dependency `{dependency_name}` ({full_path:?}) for manifest {manifest_path:?}"
                        )
                        })?;

            dependency_scopes.push(new_scope_id);
        }

        self.items
            .get_mut(scope_id)?
            .dependencies
            .append(&mut dependency_scopes);

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

            let value = items.add(Component::from_ember(make_item_data(item), component));
            items
                .get_or_create_scope_mut(manifest_path.clone(), scope_id, scope_path)?
                .components
                .insert(item.as_snake()?.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_ember(make_item_data(item), concept));
            items
                .get_or_create_scope_mut(manifest_path.clone(), scope_id, scope_path)?
                .concepts
                .insert(item.as_snake()?.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_ember(make_item_data(item), message));
            items
                .get_or_create_scope_mut(manifest_path.clone(), scope_id, scope_path)?
                .messages
                .insert(item.as_pascal()?.clone(), value);
        }

        for (segment, enum_ty) in manifest.enums.iter() {
            let enum_id = items.add(Type::from_ember_enum(
                make_item_data(&Identifier::from(segment.clone())),
                enum_ty,
            ));
            items
                .get_mut(scope_id)?
                .types
                .insert(segment.clone(), enum_id);
        }

        visited_files.remove(&full_path);

        Ok(scope_id)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StandardDefinitions {
    pub attributes: StandardAttributes,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StandardAttributes {
    pub debuggable: ItemId<Attribute>,
    pub networked: ItemId<Attribute>,
    pub resource: ItemId<Attribute>,
    pub maybe_resource: ItemId<Attribute>,
    pub store: ItemId<Attribute>,
    pub enum_: ItemId<Attribute>,
}

fn create_root_scope(items: &mut ItemMap) -> anyhow::Result<(ItemId<Scope>, StandardDefinitions)> {
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
            id: SnakeCaseIdentifier::default().into(),
            source: ItemSource::System,
        },
        SnakeCaseIdentifier::default(),
        None,
        None,
        true,
    ));

    for (id, pt) in primitive_component_definitions!(define_primitive_types) {
        let id = PascalCaseIdentifier::new(id)
            .map_err(anyhow::Error::msg)
            .context("standard value was not valid snake-case")?;

        let ty = Type::new(
            ItemData {
                parent_id: Some(root_scope),
                id: id.clone().into(),
                source: ItemSource::System,
            },
            TypeInner::Primitive(pt),
        );
        let item_id = items.add(ty);
        items.get_mut(root_scope)?.types.insert(id, item_id);
    }

    fn make_attribute(
        items: &mut ItemMap,
        root_scope: ItemId<Scope>,
        name: &str,
    ) -> anyhow::Result<ItemId<Attribute>> {
        let id = PascalCaseIdentifier::new(name)
            .map_err(anyhow::Error::msg)
            .context("standard value was not valid snake-case")?;
        let item_id = items.add(Attribute {
            data: ItemData {
                parent_id: Some(root_scope),
                id: id.clone().into(),
                source: ItemSource::System,
            },
        });
        items.get_mut(root_scope)?.attributes.insert(id, item_id);
        Ok(item_id)
    }

    let attributes = StandardAttributes {
        debuggable: make_attribute(items, root_scope, "Debuggable")?,
        networked: make_attribute(items, root_scope, "Networked")?,
        resource: make_attribute(items, root_scope, "Resource")?,
        maybe_resource: make_attribute(items, root_scope, "MaybeResource")?,
        store: make_attribute(items, root_scope, "Store")?,
        enum_: make_attribute(items, root_scope, "Enum")?,
    };

    let standard_definitions = StandardDefinitions { attributes };
    Ok((root_scope, standard_definitions))
}
