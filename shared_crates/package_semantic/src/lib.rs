use std::{
    cell::Ref,
    collections::{HashMap, HashSet},
    path::PathBuf,
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
pub use package::{Dependency, Package, PackageLocator, PackageSource};

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
use url::Url;
use util::read_file;
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
    pub standard_definitions: StandardDefinitions,
}
impl Semantic {
    pub async fn new() -> anyhow::Result<Self> {
        let mut items = ItemMap::default();
        let (root_scope_id, standard_definitions) = create_root_scope(&mut items)?;

        let mut semantic = Self {
            schema: HashMap::from_iter(ambient_schema::FILES.iter().copied()),
            items,
            root_scope_id,
            packages: HashMap::new(),
            standard_definitions,
        };

        semantic
            .add_package(PackageSource::Ambient(PathBuf::default()))
            .await?;

        Ok(semantic)
    }

    #[async_recursion]
    pub async fn add_package(&mut self, source: PackageSource) -> anyhow::Result<ItemId<Package>> {
        let (manifest, manifest_url) = source.get_manifest(&self.schema).await?;

        let locator = PackageLocator::from_manifest(&manifest, source.clone());
        if let Some(id) = self.packages.get(&locator) {
            return Ok(*id);
        }

        let build_metadata = if let Some(url) = manifest_url.as_ref() {
            Some(BuildMetadata::parse(
                &read_file(&url.join(BuildMetadata::FILENAME)?).await?,
            )?)
        } else {
            None
        };

        let mut dependencies = HashMap::new();
        for (dependency_name, dependency) in &manifest.dependencies {
            let Some(manifest_url) = manifest_url.as_ref() else {
                anyhow::bail!("the manifest URL is empty; are you trying to add dependencies to the Ambient schema?");
            };

            // path takes precedence over url
            let source = match (&dependency.path, &dependency.url) {
                (None, None) => {
                    anyhow::bail!("dependency {dependency_name} has no sources specified")
                }
                (Some(path), _) => PackageSource::Url(if path.is_relative() {
                    manifest_url.join(&path.to_string_lossy())?
                } else {
                    Url::from_file_path(path).unwrap()
                }),
                (_, Some(url)) => PackageSource::Url(url.clone()),
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
            .add_scope_from_manifest_with_includes(None, &manifest, source.clone())
            .await?;

        let package = Package {
            data: ItemData {
                parent_id: None,
                id: locator.id.clone().into(),
                source: match source {
                    PackageSource::Ambient(_) => ItemSource::Ambient,
                    PackageSource::Path(_) | PackageSource::Url(_) => ItemSource::User,
                },
            },
            source,
            manifest,
            build_metadata,
            dependencies,
            scope_id,
        };

        Ok(self.items.add(package))
    }

    pub fn resolve(&mut self) -> anyhow::Result<()> {
        let mut ids = HashSet::new();
        for locator in self.packages.keys() {
            if !ids.insert(locator.id.clone()) {
                anyhow::bail!("duplicate package found with ID: {} (the system does not currently support having an package with multiple different versions)", locator.id);
            }
        }

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
    #[async_recursion]
    async fn add_scope_from_manifest_with_includes(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        manifest: &Manifest,
        source: PackageSource,
    ) -> anyhow::Result<ItemId<Scope>> {
        let includes = manifest.package.includes.clone();
        let scope_id =
            self.add_scope_from_manifest_without_includes(parent_id, manifest, source.clone())?;

        for include in &includes {
            anyhow::ensure!(
                include.extension().is_some(),
                "include {} must have a path",
                include.display()
            );

            let include_source = source.join(include.parent().context("include has no parent")?)?;
            let include_manifest = include_source.get_manifest(&self.schema).await?.0;
            let include_scope_id = self
                .add_scope_from_manifest_with_includes(
                    Some(scope_id),
                    &include_manifest,
                    include_source,
                )
                .await?;
            let id = self.items.get(include_scope_id)?.data().id.clone();

            self.items
                .get_mut(scope_id)?
                .scopes
                .insert(id.as_snake()?.clone(), include_scope_id);
        }

        Ok(scope_id)
    }

    fn add_scope_from_manifest_without_includes(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        manifest: &Manifest,
        source: PackageSource,
    ) -> anyhow::Result<ItemId<Scope>> {
        let item_source = match source {
            PackageSource::Ambient(_) => ItemSource::Ambient,
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
                .get_or_create_scope_mut(scope_id, scope_path)?
                .components
                .insert(item.as_snake()?.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Concept::from_package(make_item_data(item), concept));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .concepts
                .insert(item.as_snake()?.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let value = items.add(Message::from_package(make_item_data(item), message));
            items
                .get_or_create_scope_mut(scope_id, scope_path)?
                .messages
                .insert(item.as_pascal()?.clone(), value);
        }

        for (segment, enum_ty) in manifest.enums.iter() {
            let enum_id = items.add(Type::from_package_enum(
                make_item_data(&Identifier::from(segment.clone())),
                enum_ty,
            ));
            items
                .get_mut(scope_id)?
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

fn create_root_scope(items: &mut ItemMap) -> anyhow::Result<(ItemId<Scope>, StandardDefinitions)> {
    macro_rules! define_primitive_types {
        ($(($value:ident, $_type:ty)),*) => {
            [
                $((stringify!($value), PrimitiveType::$value)),*
            ]
        };
    }

    let root_scope = items.add(Scope::new(ItemData {
        parent_id: None,
        id: SnakeCaseIdentifier::default().into(),
        source: ItemSource::System,
    }));

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
