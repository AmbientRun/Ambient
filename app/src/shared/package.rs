use std::collections::HashMap;

use ambient_ecs::{
    ComponentRegistry, ExternalComponentAttributes, ExternalComponentDesc, PrimitiveComponentType,
    World,
};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_network::ServerWorldExt;
use ambient_package_semantic::{
    Item, ItemId, ItemSource, Package, PrimitiveType, RetrievableFile, Semantic, TypeInner,
};

pub async fn add(
    world: Option<&mut World>,
    semantic: &mut Semantic,
    url: &AbsAssetUrl,
) -> anyhow::Result<ItemId<Package>> {
    let id = semantic
        .add_package(RetrievableFile::Url(url.0.clone()))
        .await?;
    // HACK: think about how this could be supplied in the right place
    if let Some(world) = world {
        let package_name_to_url = world
            .synced_resource_mut(ambient_package_semantic_native::package_name_to_url())
            .unwrap();

        for id in semantic.items.scope_and_dependencies(id) {
            let package = semantic.items.get(id);
            let package_id = package.data.id.to_string();

            // HACK: assume that any local urls are in the build directory.
            // I think this should generally be true, but something to watch out for.
            let asset_url = if let Some(url) = package.source.as_remote_url() {
                // Remove the manifest from the URL
                url.join("./")?
            } else {
                AbsAssetUrl::from_asset_key(&package_id)?.0
            };

            package_name_to_url.insert(package_id.clone(), asset_url.to_string());
        }
    }

    semantic.resolve()?;
    ComponentRegistry::get_mut().add_external(all_defined_components(semantic)?);

    Ok(id)
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
