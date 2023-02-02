use std::collections::HashMap;

use elements_ecs::{ComponentRegistry, PrimitiveComponentType};
use serde::Deserialize;

pub(crate) fn init() -> anyhow::Result<()> {
    elements_app::init_all_components();
    elements_network::init_all_components();
    elements_physics::init_all_components();
    elements_scripting_host::shared::init_components();
    elements_decals::init_components();
    elements_world_audio::init_components();
    elements_primitives::init_components();

    tilt_runtime_core::init_all_components();
    crate::player::init_all_components();

    // Temporary: this information should move to the ECS through attributes
    load_from_toml(include_str!("../tilt.toml"))?;

    Ok(())
}

fn load_from_toml(manifest: &str) -> anyhow::Result<()> {
    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum ComponentTypeToml {
        String(String),
        ContainerType {
            #[serde(rename = "type")]
            type_: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            element_type: Option<String>,
        },
    }
    impl TryFrom<ComponentTypeToml> for PrimitiveComponentType {
        type Error = &'static str;

        fn try_from(value: ComponentTypeToml) -> Result<Self, Self::Error> {
            match value {
                ComponentTypeToml::String(ty) => PrimitiveComponentType::try_from(ty.as_str()),
                ComponentTypeToml::ContainerType { type_, element_type } => {
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
    }

    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    struct ComponentToml {
        name: String,
        description: String,
        #[serde(rename = "type")]
        type_: ComponentTypeToml,
    }

    #[derive(Deserialize, Debug)]
    struct Manifest {
        components: HashMap<String, ComponentToml>,
    }

    let manifest: Manifest = toml::from_str(&manifest)?;
    ComponentRegistry::get_mut().add_external_from_iterator(
        manifest
            .components
            .into_iter()
            .map(|(id, component)| Ok((id, PrimitiveComponentType::try_from(component.type_).map_err(|e| anyhow::anyhow!("{e}"))?)))
            .collect::<anyhow::Result<Vec<_>>>()?
            .into_iter(),
    );

    Ok(())
}
