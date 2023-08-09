use std::{collections::HashMap, path::Path};

use ambient_ecs::{
    ComponentRegistry, ExternalComponentAttributes, ExternalComponentDesc, PrimitiveComponentType,
    World,
};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_network::ServerWorldExt;
use ambient_project::{BuildMetadata, Manifest};
use ambient_project_semantic::{Item, ItemId, PrimitiveType, Scope, Semantic, TypeInner};

pub async fn add(
    world: Option<&mut World>,
    semantic: &mut Semantic,
    path: &Path,
) -> anyhow::Result<ItemId<Scope>> {
    let id = semantic.add_ember(path).await?;
    // HACK: think about how this could be supplied in the right place
    if let Some(world) = world {
        let ember_name_to_url = world
            .synced_resource_mut(ambient_ember_semantic_native::ember_name_to_url())
            .unwrap();

        for id in semantic.items.scope_and_dependencies(id) {
            let item = semantic.items.get(id)?;
            let original_id = item.original_id.to_string();

            ember_name_to_url.insert(
                original_id.clone(),
                AbsAssetUrl::from_asset_key(original_id)?.0.to_string(),
            );
        }
    }

    finish_add(semantic, id)
}

/// HACK! Temporary to enable remote single-ember deployments.
pub async fn add_parsed_manifest(
    semantic: &mut Semantic,
    manifest: &Manifest,
    build_metadata: BuildMetadata,
) -> anyhow::Result<ItemId<Scope>> {
    let id = semantic.add_ember_manifest(manifest).await?;
    semantic.items.get_mut(id)?.build_metadata = Some(build_metadata);
    finish_add(semantic, id)
}

fn finish_add(semantic: &mut Semantic, id: ItemId<Scope>) -> anyhow::Result<ItemId<Scope>> {
    semantic.resolve()?;
    ComponentRegistry::get_mut().add_external(all_defined_components(semantic)?);

    Ok(id)
}

fn all_defined_components(semantic: &Semantic) -> anyhow::Result<Vec<ExternalComponentDesc>> {
    let items = &semantic.items;
    let root_scope = &semantic.root_scope();

    let type_map = {
        let mut type_map = HashMap::new();

        // First pass: add all root-level primitive types
        for type_id in root_scope.types.values() {
            let type_ = items.get(*type_id).expect("type id not in items");
            if let TypeInner::Primitive(pt) = type_.inner {
                let ty = primitive_type_to_primitive_component_type(pt);
                type_map.insert(*type_id, ty);
                type_map.insert(items.get_vec_id(*type_id), ty.to_vec_type().unwrap());
                type_map.insert(items.get_option_id(*type_id), ty.to_option_type().unwrap());
            }
        }

        // Second pass: traverse the type graph and add all enums
        root_scope.visit_recursive(items, |scope| {
            for type_id in scope.types.values() {
                let type_ = items.get(*type_id).expect("type id not in items");
                if let TypeInner::Enum { .. } = type_.inner {
                    type_map.insert(*type_id, PrimitiveComponentType::U32);
                }
            }
            Ok(())
        })?;

        type_map
    };

    let mut components = vec![];
    root_scope.visit_recursive(items, |scope| {
        for id in scope.components.values().copied() {
            let component = items.get(id)?;

            let attributes: Vec<_> = component
                .attributes
                .iter()
                .map(|id| {
                    let attr = items.get(id.as_resolved().unwrap_or_else(|| {
                        panic!(
                            "attribute id {:?} not resolved in component {:?}",
                            id, component
                        )
                    }))?;
                    Ok(attr.data().id.to_string())
                })
                .collect::<anyhow::Result<_>>()?;

            components.push(ExternalComponentDesc {
                path: items.fully_qualified_display_path(&*component, true, None, None)?,
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
