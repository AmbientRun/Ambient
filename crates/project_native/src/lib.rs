use ambient_ecs::{
    ExternalComponentAttributes, ExternalComponentDesc, ExternalComponentFlagAttributes,
    PrimitiveComponentType,
};

use ambient_project::{ComponentType, IdentifierPathBuf, Manifest};

pub fn all_defined_components(
    manifest: &Manifest,
    global_namespace: bool,
) -> Result<Vec<ExternalComponentDesc>, &'static str> {
    let project_path: Vec<_> = if global_namespace {
        vec![]
    } else {
        manifest
            .project
            .organization
            .iter()
            .chain(std::iter::once(&manifest.project.id))
            .cloned()
            .collect()
    };

    manifest
        .components
        .iter()
        .filter_map(|(id, component)| Some((id, component.other()?)))
        .map(|(id, component)| {
            let full_path = IdentifierPathBuf::from_iter(
                project_path.iter().chain(id.as_path().iter()).cloned(),
            );
            Ok(ExternalComponentDesc {
                path: full_path.to_string(),
                ty: component_type_to_primitive(&component.type_)?,
                attributes: ExternalComponentAttributes {
                    name: component.name.clone(),
                    description: component.description.clone(),
                    flags: ExternalComponentFlagAttributes::from_iter(
                        component.attributes.iter().map(|s| s.as_str()),
                    ),
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn component_type_to_primitive(ty: &ComponentType) -> Result<PrimitiveComponentType, &'static str> {
    match ty {
        ComponentType::String(ty) => PrimitiveComponentType::try_from(ty.as_str()),
        ComponentType::ContainerType {
            type_,
            element_type,
        } => {
            let element_ty = element_type
                .as_deref()
                .map(PrimitiveComponentType::try_from)
                .transpose()?;
            match element_ty {
                Some(element_ty) => match type_.as_str() {
                    "Vec" => element_ty
                        .to_vec_type()
                        .ok_or("invalid element type for Vec"),
                    "Option" => element_ty
                        .to_option_type()
                        .ok_or("invalid element type for Option"),
                    _ => Err("invalid container type"),
                },
                None => PrimitiveComponentType::try_from(type_.as_str()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ambient_shared_types::primitive_component_definitions;

    use crate::{component_type_to_primitive, ComponentType};

    #[test]
    fn can_convert_component_types() {
        use ambient_ecs::PrimitiveComponentType as PCT;
        use ComponentType as CT;

        fn test_type(ty: &str, pct_raw: PCT, pct_vec: PCT, pct_option: PCT) {
            fn str_ty(ty: &str) -> CT {
                CT::String(ty.to_string())
            }

            fn ct_str_ty(ty: &str) -> CT {
                CT::ContainerType {
                    type_: ty.to_string(),
                    element_type: None,
                }
            }

            fn ct_ty(ct: &str, ty: &str) -> CT {
                CT::ContainerType {
                    type_: ct.to_string(),
                    element_type: Some(ty.to_string()),
                }
            }

            assert_eq!(component_type_to_primitive(&str_ty(ty)), Ok(pct_raw));
            assert_eq!(component_type_to_primitive(&ct_str_ty(ty)), Ok(pct_raw));
            assert_eq!(component_type_to_primitive(&ct_ty("Vec", ty)), Ok(pct_vec));
            assert_eq!(
                component_type_to_primitive(&ct_ty("Option", ty)),
                Ok(pct_option)
            );
        }

        macro_rules! make_test_cases {
            ($(($value:ident, $_:ty)),*) => {
                paste::paste! {
                    $(test_type(
                        stringify!($value),
                        PCT::$value,
                        PCT::[<Vec $value>],
                        PCT::[<Option $value>],
                    );)*
                }
            };
        }

        primitive_component_definitions!(make_test_cases);
    }
}
