use std::{
    collections::{HashMap, HashSet},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use ambient_cb::Cb;
use ambient_ecs::{
    components, generated::app::components::name as app_name,
    generated::wasm::components::module_enabled, query, ComponentRegistry, Entity, EntityId,
    ExternalComponentAttributes, ExternalComponentDesc, Networked, PrimitiveComponentType,
    Resource, SystemGroup, World,
};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_network::ServerWorldExt;
use ambient_package_semantic::{
    Item, ItemId, ItemSource, LocalOrRemote, Package, PrimitiveType, RetrievableFile, Semantic,
    TypeInner,
};

pub use ambient_ecs::generated::package::components::*;
use thiserror::Error;
use tokio::sync::Mutex;

components!("semantic", {
    @[Resource]
    semantic: Arc<Mutex<Semantic>>,

    @[Resource, Networked]
    package_id_to_package_entity: HashMap<String, EntityId>,

    /// Used to spawn the WASM modules for each package
    @[Resource]
    wasm_spawn: WasmSpawn,
});

pub type WasmSpawn =
    Cb<dyn Fn(&mut World, WasmSpawnRequest) -> anyhow::Result<WasmSpawnResponse> + Sync + Send>;
#[derive(Debug)]
pub struct WasmSpawnRequest {
    pub client_modules: Vec<(AbsAssetUrl, bool)>,
    pub server_modules: Vec<(AbsAssetUrl, bool)>,
}
#[derive(Debug, Default)]
pub struct WasmSpawnResponse {
    pub client_modules: Vec<EntityId>,
    pub server_modules: Vec<EntityId>,
}

pub fn world_semantic(world: &World) -> Arc<Mutex<Semantic>> {
    world.resource(semantic()).clone()
}

pub async fn initialize(
    world: &mut World,
    main_package_path: &AbsAssetUrl,
    wasm_spawn: WasmSpawn,
) -> anyhow::Result<()> {
    world.add_resource(self::wasm_spawn(), wasm_spawn);
    world.add_resource(
        self::semantic(),
        Arc::new(Mutex::new(
            ambient_package_semantic::Semantic::new(false).await?,
        )),
    );

    add(world, &main_package_path.push("ambient.toml")?).await?;

    Ok(())
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "package",
        vec![
            query((enabled().changed(), client_modules(), server_modules())).to_system(
                |q, world, qs, _| {
                    for (_, (enabled, client_modules, server_modules)) in
                        q.collect_cloned(world, qs)
                    {
                        for id in client_modules.into_iter().chain(server_modules) {
                            world.add_component(id, module_enabled(), enabled).unwrap();
                        }
                    }
                },
            ),
        ],
    )
}

pub async fn add(world: &mut World, package_url: &AbsAssetUrl) -> anyhow::Result<ItemId<Package>> {
    let semantic = world_semantic(world);
    // We must use a Tokio mutex as we need to be able to hold onto the semantic through
    // the await point, and the semantic has RefCells inside of it.
    let mut semantic = semantic.lock().await;

    let package_item_id =
        add_to_semantic_and_register_components(&mut semantic, package_url).await?;

    let package_id_spawned = world
        .synced_resource(package_id_to_package_entity())
        .unwrap()
        .keys()
        .cloned()
        .collect::<HashSet<_>>();

    // Use the topologically sorted queue to construct a dict of which packages should be on by default.
    // Assume all are on by default, and then update their state based on what packages "closer to the root"
    // state. The last element should be the root.
    let package_id_to_enabled = {
        let queue = semantic.items.scope_and_dependencies(package_item_id);

        let mut package_id_to_enabled = queue
            .iter()
            .map(|&id| (id, true))
            .collect::<HashMap<_, _>>();

        for &package_id in &queue {
            let package = semantic.items.get(package_id);

            for dependency in package.dependencies.values() {
                if let Some(enabled) = dependency.enabled {
                    package_id_to_enabled.insert(dependency.id, enabled);
                }
            }
        }

        package_id_to_enabled
    };

    // Spawn all of the packages.
    for package_item_id in semantic.items.scope_and_dependencies(package_item_id) {
        let package = semantic.items.get(package_item_id);
        let package_id = package.data.id.to_string();

        if package_id_spawned.contains(&package_id) {
            continue;
        }

        let base_asset_url = match package.source.as_local_or_remote().unwrap() {
            LocalOrRemote::Local(_) => {
                // HACK: assume that any local urls are in the build directory.
                // I think this should generally be true, but something to watch out for.
                AbsAssetUrl::from_asset_key(&package_id)?.0
            }
            LocalOrRemote::Remote(url) => {
                // Remove the manifest from the URL
                url.join("./")?
            }
        };

        let enabled = package_id_to_enabled
            .get(&package_item_id)
            .copied()
            .unwrap_or(true);

        let manifest = &package.manifest;
        let mut entity = Entity::new()
            .with(app_name(), format!("Package {}", manifest.package.name))
            .with(self::is_package(), ())
            .with(self::enabled(), enabled)
            .with(self::id(), package_id.clone())
            .with(self::name(), manifest.package.name.clone())
            .with(self::version(), manifest.package.version.to_string())
            .with(self::authors(), manifest.package.authors.clone())
            .with(self::asset_url(), base_asset_url.to_string());
        if let Some(description) = &manifest.package.description {
            entity.set(self::description(), description.clone());
        }
        if let Some(repository) = &manifest.package.repository {
            entity.set(self::repository(), repository.clone());
        }
        let entity = entity.spawn(world);
        world
            .synced_resource_mut(package_id_to_package_entity())
            .unwrap()
            .insert(package_id.clone(), entity);

        if let Some(metadata) = &package.build_metadata {
            let asset_url = AbsAssetUrl(base_asset_url.clone());

            let wasm_spawn = world.resource(self::wasm_spawn()).clone();
            let wasm = (wasm_spawn)(
                world,
                WasmSpawnRequest {
                    client_modules: metadata
                        .client_component_paths
                        .iter()
                        .map(|m| Ok((asset_url.push(m)?, enabled)))
                        .collect::<Result<Vec<_>, url::ParseError>>()?,
                    server_modules: metadata
                        .server_component_paths
                        .iter()
                        .map(|m| Ok((asset_url.push(m)?, enabled)))
                        .collect::<Result<Vec<_>, url::ParseError>>()?,
                },
            )?;

            world.add_component(entity, self::client_modules(), wasm.client_modules)?;
            world.add_component(entity, self::server_modules(), wasm.server_modules)?;
        };
    }

    Ok(package_item_id)
}

pub async fn add_to_semantic_and_register_components(
    semantic: &mut Semantic,
    url: &AbsAssetUrl,
) -> anyhow::Result<ItemId<Package>> {
    let id = semantic
        .add_package(RetrievableFile::Url(url.0.clone()), None)
        .await?;

    semantic.resolve()?;
    ComponentRegistry::get_mut().add_external(all_defined_components(semantic)?);

    Ok(id)
}

#[derive(Error, Debug)]
pub enum FilePathError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

/// Returns the path for the given file in the given package, or returns a global path
/// if that package doesn't have an associated URL.
///
/// Note that `path` is relative to the root of the package's build directory, so an
/// asset will require `assets/` prefixed to the path.
pub fn file_path(
    world: &World,
    package_id: &str,
    path: &Path,
) -> Result<AbsAssetUrl, FilePathError> {
    let entity = world
        .synced_resource(package_id_to_package_entity())
        .unwrap()
        .get(package_id)
        .copied()
        .ok_or_else(|| FilePathError::PackageNotFound(package_id.to_string()))?;

    if let Ok(url) = world.get_cloned(entity, asset_url()) {
        Ok(AbsAssetUrl::from_str(&format!("{url}/{}", path.display()))?)
    } else {
        Ok(AbsAssetUrl::from_asset_key(path.to_string_lossy())?)
    }
}

fn all_defined_components(semantic: &Semantic) -> anyhow::Result<Vec<ExternalComponentDesc>> {
    let items = &semantic.items;
    let root_scope = semantic.root_scope();

    let type_map = {
        let mut type_map = HashMap::new();

        // First pass: add all root-level primitive types
        for type_id in root_scope.types.values() {
            let type_ = items.get(*type_id);
            if let TypeInner::Primitive(pt) = type_.inner {
                let ty = primitive_type_to_primitive_component_type(pt);
                type_map.insert(*type_id, ty);
                type_map.insert(items.get_vec_id(*type_id), ty.to_vec_type().unwrap());
                type_map.insert(items.get_option_id(*type_id), ty.to_option_type().unwrap());
            }
        }

        // Second pass: traverse the type graph and add all enums
        for package_id in semantic.packages.values() {
            let package = items.get(*package_id);
            let scope = items.get(package.scope_id);
            scope.visit_recursive(items, |scope| {
                for type_id in scope.types.values() {
                    let type_ = items.get(*type_id);
                    if let TypeInner::Enum { .. } = type_.inner {
                        type_map.insert(*type_id, PrimitiveComponentType::U32);
                    }
                }
                Ok(())
            })?;
        }

        type_map
    };

    let mut components = vec![];
    for package_id in semantic.packages.values() {
        let package = items.get(*package_id);
        let scope = items.get(package.scope_id);
        scope.visit_recursive(items, |scope| {
            for id in scope.components.values().copied() {
                let component = items.get(id);

                if component.data.source != ItemSource::User {
                    continue;
                }

                let attributes: Vec<_> = component
                    .attributes
                    .iter()
                    .map(|id| {
                        let attr = items.get(id.as_resolved().unwrap_or_else(|| {
                            panic!(
                                "attribute id {:?} not resolved in component {:?}",
                                id, component
                            )
                        }));
                        Ok(attr.data().id.to_string())
                    })
                    .collect::<anyhow::Result<_>>()?;

                components.push(ExternalComponentDesc {
                    path: items.fully_qualified_display_path(&*component, None, None),
                    ty: type_map[&component.type_.as_resolved().unwrap_or_else(|| {
                        panic!(
                            "type id {:?} not resolved in component {:?}",
                            component.type_, component
                        )
                    })],
                    name: component.name.clone(),
                    description: component.description.clone(),
                    attributes: ExternalComponentAttributes::from_iter(
                        attributes.iter().map(|s| s.as_str()),
                    ),
                });
            }
            Ok(())
        })?;
    }
    Ok(components)
}

fn primitive_type_to_primitive_component_type(pt: PrimitiveType) -> PrimitiveComponentType {
    macro_rules! convert {
        ($(($value:ident, $_type:ty)),*) => {
            match pt {
                $(PrimitiveType::$value => PrimitiveComponentType::$value,)*
            }
        };
    }

    ambient_shared_types::primitive_component_definitions!(convert)
}
