use std::collections::HashMap;

use elements_ecs::{ComponentRegistry, PrimitiveComponentType};
use elements_project::{ComponentType, Manifest};
use serde::Deserialize;

pub(crate) fn init() -> anyhow::Result<()> {
    elements_app::init_all_components();
    elements_network::init_all_components();
    elements_physics::init_all_components();
    elements_scripting_host::shared::init_components();
    elements_decals::init_components();
    elements_world_audio::init_components();
    elements_primitives::init_components();
    elements_project::init_components();

    tilt_runtime_core::init_all_components();
    crate::player::init_all_components();

    // Temporary: this information should move to the ECS through attributes
    load_from_toml(&Manifest::from_str(include_str!("../tilt.toml"))?)?;

    Ok(())
}

fn load_from_toml(manifest: &Manifest) -> anyhow::Result<()> {
    ComponentRegistry::get_mut().add_external_from_iterator(
        manifest
            .components
            .iter()
            .map(|(id, component)| Ok((id.to_owned(), convert_manifest_type_to_primitive_type(&component.type_)?)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e: &str| anyhow::anyhow!("{e}"))?
            .into_iter(),
    );

    Ok(())
}

fn convert_manifest_type_to_primitive_type(ty: &ComponentType) -> Result<PrimitiveComponentType, &'static str> {
    match ty {
        ComponentType::String(ty) => PrimitiveComponentType::try_from(ty.as_str()),
        ComponentType::ContainerType { type_, element_type } => {
            let element_ty = element_type.as_deref().map(PrimitiveComponentType::try_from).transpose()?.map(Box::new);
            match element_ty {
                Some(element_ty) => match type_.as_str() {
                    "Vec" => Ok(PrimitiveComponentType::Vec { variants: element_ty }),
                    "Option" => Ok(PrimitiveComponentType::Option { variants: element_ty }),
                    _ => Err("invalid container type"),
                },
                None => PrimitiveComponentType::try_from(type_.as_str()),
            }
        }
    }
}
