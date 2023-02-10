use std::collections::HashMap;

use kiwi_ecs::primitive_component_definitions;

use crate::{Component, ComponentType, Identifier, IdentifierPath, Manifest, Project, Version, VersionError};

#[test]
fn can_parse_tictactoe_toml() {
    const TOML: &str = r#"
    [project]
    id = "tictactoe"
    name = "Tic Tac Toe"
    version = "0.0.1"

    [components]
    cell = { type = "I32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["Store"] }
    "#;

    assert_eq!(
        Manifest::parse(TOML),
        Ok(Manifest {
            project: Project {
                id: Identifier::new("tictactoe").unwrap(),
                name: Some("Tic Tac Toe".to_string()),
                version: Version::new(0, 0, 1),
                description: None,
                authors: vec![],
                organization: None
            },
            components: HashMap::from_iter([(
                IdentifierPath::new("cell").unwrap(),
                Component {
                    name: "Cell".to_string(),
                    description: "The ID of the cell this player is in".to_string(),
                    type_: ComponentType::String("I32".to_string()),
                    attributes: vec!["Store".to_string()]
                }
            )])
        })
    )
}

#[test]
fn can_validate_identifiers() {
    use Identifier as I;
    use IdentifierPath as IP;

    assert_eq!(I::new(""), Err("identifier must not be empty"));
    assert_eq!(I::new("5asd"), Err("identifier must start with a lowercase ASCII character"));
    assert_eq!(I::new("_asd"), Err("identifier must start with a lowercase ASCII character"));
    assert_eq!(I::new("mY_COOL_COMPONENT"), Err("identifier must be snake-case ASCII"));
    assert_eq!(I::new("cool_component!"), Err("identifier must be snake-case ASCII"));
    assert_eq!(I::new("cool-component"), Err("identifier must be snake-case ASCII"));

    assert_eq!(I::new("cool_component"), Ok(I("cool_component".to_string())));
    assert_eq!(I::new("cool_component_00"), Ok(I("cool_component_00".to_string())));

    assert_eq!(IP::new("my::cool_component_00"), Ok(IP(vec![I("my".to_string()), I("cool_component_00".to_string())])));
}

#[test]
fn can_parse_versions() {
    use Version as V;

    assert_eq!(V::new_from_str("1"), Ok(V::new(1, 0, 0)));
    assert_eq!(V::new_from_str("1.0"), Ok(V::new(1, 0, 0)));
    assert_eq!(V::new_from_str("1.0.0"), Ok(V::new(1, 0, 0)));
    assert_eq!(V::new_from_str("1.2.3"), Ok(V::new(1, 2, 3)));

    assert_eq!(V::new_from_str(""), Err(VersionError::TooFewComponents));
    assert_eq!(V::new_from_str("0.0.0"), Err(VersionError::AllZero));
    assert!(matches!(V::new_from_str("1.2.3patch"), Err(VersionError::InvalidNumber(_))));
    assert_eq!(V::new_from_str("1.2.3.4"), Err(VersionError::TooManyComponents));
}

#[test]
fn can_convert_component_types() {
    use kiwi_ecs::PrimitiveComponentType as PCT;
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
