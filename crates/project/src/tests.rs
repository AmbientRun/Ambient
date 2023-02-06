use std::collections::HashMap;

use elements_ecs::primitive_component_definitions;

use crate::{Component, ComponentType, Manifest, Project};

#[test]
fn can_parse_tictactoe_toml() {
    const TOML: &str = r#"
    [project]
    name = "tictactoe"
    version = "0.0.1"

    [components]
    cell = { type = "I32", name = "Cell", description = "The ID of the cell this player is in" }
    "#;

    assert_eq!(
        Manifest::parse(TOML),
        Ok(Manifest {
            project: Project {
                name: "tictactoe".to_string(),
                version: "0.0.1".to_string(),
                description: None,
                authors: vec![],
                organization: None
            },
            components: HashMap::from_iter([(
                "cell".to_string(),
                Component {
                    name: "Cell".to_string(),
                    description: "The ID of the cell this player is in".to_string(),
                    type_: ComponentType::String("I32".to_string())
                }
            )])
        })
    )
}

#[test]
fn can_convert_component_types() {
    use elements_ecs::PrimitiveComponentType as PCT;
    use ComponentType as CT;

    fn test_type(ty: &str, pct_raw: PCT, pct_vec: PCT, pct_option: PCT) {
        fn str_ty(ty: &str) -> CT {
            CT::String(ty.to_string())
        }

        fn ct_str_ty(ty: &str) -> CT {
            CT::ContainerType { type_: ty.to_string(), element_type: None }
        }

        fn ct_ty(ct: &str, ty: &str) -> CT {
            CT::ContainerType { type_: ct.to_string(), element_type: Some(ty.to_string()) }
        }

        assert_eq!(PCT::try_from(&str_ty(ty)), Ok(pct_raw));
        assert_eq!(PCT::try_from(&ct_str_ty(ty)), Ok(pct_raw));
        assert_eq!(PCT::try_from(&ct_ty("Vec", ty)), Ok(pct_vec));
        assert_eq!(PCT::try_from(&ct_ty("Option", ty)), Ok(pct_option));
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
