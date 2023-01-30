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

    crate::player::init_all_components();

    load_from_toml(include_str!("components.toml"))?;

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
    impl ComponentTypeToml {
        fn type_str(&self) -> (&str, Option<&str>) {
            match self {
                ComponentTypeToml::String(type_) => (type_.as_str(), None),
                ComponentTypeToml::ContainerType { type_, element_type } => (type_.as_str(), element_type.as_deref()),
            }
        }
    }
    impl TryFrom<ComponentTypeToml> for PrimitiveComponentType {
        type Error = &'static str;

        fn try_from(value: ComponentTypeToml) -> Result<Self, Self::Error> {
            let (ty, element_ty) = value.type_str();
            match ty {
                "Vec" => Ok(PrimitiveComponentType::Vec {
                    variants: Box::new(PrimitiveComponentType::try_from(element_ty.ok_or("expected element_type")?)?),
                }),
                "Option" => Ok(PrimitiveComponentType::Option {
                    variants: Box::new(PrimitiveComponentType::try_from(element_ty.ok_or("expected element_type")?)?),
                }),
                _ => PrimitiveComponentType::try_from(ty),
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

    let components: HashMap<String, ComponentToml> = toml::from_str(manifest)?;
    let components: Vec<_> = components
        .into_iter()
        .map(|(id, component)| Ok((id, PrimitiveComponentType::try_from(component.type_)?)))
        .collect::<Result<_, _>>()
        .map_err(|e: &'static str| anyhow::anyhow!("{e}"))?;
    ComponentRegistry::get_mut().add_external_from_iterator(components.into_iter());

    Ok(())
}
