use std::{
    cell::Ref,
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Context as AnyhowContext;
use async_recursion::async_recursion;

use ambient_package::{
    BuildMetadata, Identifier, Manifest, PascalCaseIdentifier, SnakeCaseIdentifier,
};
use ambient_shared_types::primitive_component_definitions;

mod scope;
pub use scope::{Context, Scope};

mod package;
pub use package::{Dependency, Package, PackageLocator, RetrievableFile};

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

mod util;

pub type Schema = HashMap<&'static str, &'static str>;

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    pub schema: Schema,
    pub items: ItemMap,
    pub root_scope_id: ItemId<Scope>,
    pub packages: HashMap<PackageLocator, ItemId<Package>>,
    pub ambient_package_id: ItemId<Package>,
    pub standard_definitions: StandardDefinitions,
    ignore_local_dependencies: bool,
}
impl Semantic {
    pub async fn new(ignore_local_dependencies: bool) -> anyhow::Result<Self> {
        let mut items = ItemMap::default();
        let (root_scope_id, standard_definitions) = create_root_scope(&mut items)?;

        let mut semantic = Self {
            schema: HashMap::from_iter(ambient_schema::FILES.iter().copied()),
            items,
            root_scope_id,
            packages: HashMap::new(),
            ambient_package_id: ItemId::empty_you_should_really_initialize_this(),
            standard_definitions,
            ignore_local_dependencies,
        };

        semantic.ambient_package_id = semantic
            .add_package(RetrievableFile::Ambient(PathBuf::from("ambient.toml")))
            .await?;

        Ok(semantic)
    }

    #[cfg_attr(not(target_os = "unknown"), async_recursion)]
    #[cfg_attr(target_os = "unknown", async_recursion(?Send))]
    pub async fn add_package(
        &mut self,
        retrievable_manifest: RetrievableFile,
    ) -> anyhow::Result<ItemId<Package>> {
        let manifest = Manifest::parse(&retrievable_manifest.get(&self.schema).await?)?;

        let locator = PackageLocator::from_manifest(&manifest, retrievable_manifest.clone());
        if let Some(id) = self.packages.get(&locator) {
            return Ok(*id);
        }

        let build_metadata = retrievable_manifest
            .parent_join(Path::new(BuildMetadata::FILENAME))?
            .get(&self.schema)
            .await
            .ok()
            .map(|s| BuildMetadata::parse(&s))
            .transpose()?;

        let mut dependencies = HashMap::new();
        for (dependency_name, dependency) in &manifest.dependencies {
            // path takes precedence over url
            let source = match (
                dependency
                    .path
                    .as_ref()
                    .filter(|_| !self.ignore_local_dependencies),
                &dependency.url(),
            ) {
                (None, None) => {
                    anyhow::bail!(
                        "{locator}: dependency `{dependency_name}` has no supported sources specified (are you trying to deploy a package with a local dependency?)"
                    )
                }
                (Some(path), _) => retrievable_manifest.parent_join(&path.join("ambient.toml"))?,
                (_, Some(url)) => {
                    // ensure it is a directory
                    let mut url = url.clone();
                    if !url.path().ends_with('/') {
                        url.set_path(&format!("{}/", url.path()));
                    }
                    RetrievableFile::Url(url.join("ambient.toml")?)
                }
            };

            let dependency_id = self.add_package(source).await?;

            dependencies.insert(
                dependency_name.clone(),
                Dependency {
                    id: dependency_id,
                    enabled: dependency.enabled,
                },
            );
        }

        let scope_id = self
            .add_scope_from_manifest_with_includes(None, &manifest, retrievable_manifest.clone())
            .await?;

        {
            let mut scope = self.items.get_mut(scope_id);

            // If this is not the Ambient package, import the Ambient package
            if !matches!(retrievable_manifest, RetrievableFile::Ambient(_)) {
                let ambient_package = self.items.get(self.ambient_package_id);
                scope.imports.insert(
                    ambient_package
                        .data()
                        .id
                        .as_snake()
                        .map_err(|e| e.to_owned())?
                        .clone(),
                    self.ambient_package_id,
                );
            }

            for (name, dependency) in &dependencies {
                scope.imports.insert(name.clone(), dependency.id);
            }
        }

        let package = Package {
            data: ItemData {
                parent_id: None,
                id: locator.id.clone().into(),
                source: match retrievable_manifest {
                    RetrievableFile::Ambient(_) => ItemSource::Ambient,
                    RetrievableFile::Path(_) | RetrievableFile::Url(_) => ItemSource::User,
                },
            },
            source: retrievable_manifest,
            manifest,
            build_metadata,
            dependencies,
            scope_id,
        };

        let id = self.items.add(package);
        self.packages.insert(locator, id);
        Ok(id)
    }

    pub fn resolve(&mut self) -> anyhow::Result<()> {
        let mut seen_locators = HashMap::new();
        for locator in self.packages.keys() {
            if let Some(present) = seen_locators.insert(locator.id.clone(), locator.clone()) {
                anyhow::bail!(
                    "duplicate package found - {present} is already in the system, but {locator} was encountered ({})",
                    "the system does not currently support having an package with multiple different versions"
                );
            }
        }

        for package_id in self.packages.values().copied() {
            self.items.resolve_clone(
                &Context::new(self.root_scope_id),
                &self.standard_definitions,
                package_id,
            )?;
        }

        Ok(())
    }

    pub fn root_scope(&self) -> Ref<'_, Scope> {
        self.items.get(self.root_scope_id)
    }

    pub fn get_scope_id_by_name(&self, name: &SnakeCaseIdentifier) -> Option<ItemId<Scope>> {
        self.root_scope().scopes.get(name).copied()
    }
}
impl Semantic {
    #[cfg_attr(not(target_os = "unknown"), async_recursion)]
    #[cfg_attr(target_os = "unknown", async_recursion(?Send))]
    async fn add_scope_from_manifest_with_includes(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        manifest: &Manifest,
        source: RetrievableFile,
    ) -> anyhow::Result<ItemId<Scope>> {
        let includes = manifest.package.includes.clone();
        let scope_id =
            self.add_scope_from_manifest_without_includes(parent_id, manifest, source.clone())?;

        for include in &includes {
            anyhow::ensure!(
                include.extension().is_some(),
                "include {} must have an extension",
                include.display()
            );

            let include_source = source.parent_join(include)?;
            let include_manifest = Manifest::parse(&include_source.get(&self.schema).await?)?;
            let include_scope_id = self
                .add_scope_from_manifest_with_includes(
                    Some(scope_id),
                    &include_manifest,
                    include_source,
                )
                .await?;
            let id = self.items.get(include_scope_id).data().id.clone();

            self.items.get_mut(scope_id).scopes.insert(
                id.as_snake().map_err(|e| e.to_owned())?.clone(),
                include_scope_id,
            );
        }

        Ok(scope_id)
    }

    fn add_scope_from_manifest_without_includes(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        manifest: &Manifest,
        source: RetrievableFile,
    ) -> anyhow::Result<ItemId<Scope>> {
        let item_source = match source {
            RetrievableFile::Ambient(_) => ItemSource::Ambient,
            _ => ItemSource::User,
        };
        let scope_id = self.items.add(Scope::new(ItemData {
            parent_id,
            id: manifest.package.id.clone().into(),
            source: item_source,
        }));

        let make_item_data = |item_id: &Identifier| -> ItemData {
            ItemData {
                parent_id: Some(scope_id),
                id: item_id.clone(),
                source: item_source,
            }
        };

        let items = &mut self.items;
        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Component::from_package(make_item_data(item), component));
            items
                .get_or_create_scope_mut(scope_id, scope_path)
                .components
                .insert(item.as_snake().map_err(|e| e.to_owned())?.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_package(make_item_data(item), concept));
            items
                .get_or_create_scope_mut(scope_id, scope_path)
                .concepts
                .insert(item.as_snake().map_err(|e| e.to_owned())?.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_package(make_item_data(item), message));
            items
                .get_or_create_scope_mut(scope_id, scope_path)
                .messages
                .insert(item.as_pascal().map_err(|e| e.to_owned())?.clone(), value);
        }

        for (segment, enum_ty) in manifest.enums.iter() {
            let enum_id = items.add(Type::from_package_enum(
                make_item_data(&Identifier::from(segment.clone())),
                enum_ty,
            ));
            items
                .get_mut(scope_id)
                .types
                .insert(segment.clone(), enum_id);
        }

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

pub fn create_root_scope(
    items: &mut ItemMap,
) -> anyhow::Result<(ItemId<Scope>, StandardDefinitions)> {
    let root_scope = items.add(Scope::new(ItemData {
        parent_id: None,
        id: SnakeCaseIdentifier::default().into(),
        source: ItemSource::System,
    }));

    macro_rules! define_primitive_types {
        ($(($value:ident, $_type:ty)),*) => {
            [
                $((stringify!($value), PrimitiveType::$value)),*
            ]
        };
    }

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
        items.get_mut(root_scope).types.insert(id, item_id);
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
        items.get_mut(root_scope).attributes.insert(id, item_id);
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
