use std::collections::HashMap;

use ambient_ecs::primitive_component_definitions;

use crate::{Component, ComponentType, Concept, Identifier, IdentifierPathBuf, Manifest, Namespace, Project, Version, VersionError};

#[test]
fn can_parse_tictactoe_toml() {
    const TOML: &str = r#"
    [project]
    id = "tictactoe"
    name = "Tic Tac Toe"
    version = "0.0.1"

    [components]
    cell = { type = "I32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["Store"] }

    [concepts.cell]
    name = "Cell"
    description = "A cell object"
    [concepts.cell.components]
    cell = 0
    "#;

    assert_eq!(
        Manifest::parse(TOML),
        Ok(Manifest {
            project: Project {
                id: Identifier::new("tictactoe").unwrap(),
                name: Some("Tic Tac Toe".to_string()),
                version: Version::new(0, 0, 1, None),
                description: None,
                authors: vec![],
                organization: None
            },
            components: HashMap::from_iter([(
                IdentifierPathBuf::new("cell").unwrap(),
                Component {
                    name: "Cell".to_string(),
                    description: "The ID of the cell this player is in".to_string(),
                    type_: ComponentType::String("I32".to_string()),
                    attributes: vec!["Store".to_string()]
                }
                .into()
            )]),
            concepts: HashMap::from_iter([(
                IdentifierPathBuf::new("cell").unwrap(),
                Concept {
                    name: "Cell".to_string(),
                    description: "A cell object".to_string(),
                    extends: vec![],
                    components: HashMap::from_iter([(IdentifierPathBuf::new("cell").unwrap(), toml::Value::Integer(0))])
                }
                .into()
            )]),
        })
    )
}

#[test]
fn can_parse_manifest_with_namespaces() {
    const TOML: &str = r#"
    [project]
    id = "tictactoe"
    name = "Tic Tac Toe"
    version = "0.0.1"

    [components]
    "core" = { name = "Core", description = "" }
    "core::app" = { name = "App", description = "" }

    "core::app::main_scene" = { name = "Main Scene", description = "", type = "Empty" }
    "#;

    assert_eq!(
        Manifest::parse(TOML),
        Ok(Manifest {
            project: Project {
                id: Identifier::new("tictactoe").unwrap(),
                name: Some("Tic Tac Toe".to_string()),
                version: Version::new(0, 0, 1, None),
                description: None,
                authors: vec![],
                organization: None
            },
            components: HashMap::from_iter([
                (IdentifierPathBuf::new("core").unwrap(), Namespace { name: "Core".to_string(), description: String::new() }.into()),
                (IdentifierPathBuf::new("core::app").unwrap(), Namespace { name: "App".to_string(), description: String::new() }.into()),
                (
                    IdentifierPathBuf::new("core::app::main_scene").unwrap(),
                    Component {
                        name: "Main Scene".to_string(),
                        description: "".to_string(),
                        type_: ComponentType::String("Empty".to_string()),
                        attributes: vec![]
                    }
                    .into()
                )
            ]),
            concepts: HashMap::new(),
        })
    )
}

#[test]
fn can_generate_concepts_with_documented_namespace_from_manifest() {
    use toml::Value;

    const TOML: &str = r#"
    [project]
    id = "my_project"
    name = "My Project"
    version = "0.0.1"

    [components]
    "core::transform::rotation" = { type = "Quat", name = "Rotation", description = "" }
    "core::transform::scale" = { type = "Vec3", name = "Scale", description = "" }
    "core::transform::spherical_billboard" = { type = "Empty", name = "Spherical billboard", description = "" }
    "core::transform::translation" = { type = "Vec3", name = "Translation", description = "" }

    [concepts]
    "ns" = { name = "Namespace", description = "A Test Namespace" }
    "ns::transformable" = { name = "Transformable", description = "Can be translated, rotated and scaled.", components = {"core::transform::translation" = [0, 0, 0], "core::transform::rotation" = [0, 0, 0, 1], "core::transform::scale" = [1, 1, 1]} }
    "#;

    assert_eq!(
        Manifest::parse(TOML),
        Ok(Manifest {
            project: Project {
                id: Identifier::new("my_project").unwrap(),
                name: Some("My Project".to_string()),
                version: Version::new(0, 0, 1),
                description: None,
                authors: vec![],
                organization: None
            },
            components: HashMap::from_iter([
                (
                    IdentifierPathBuf::new("core::transform::rotation").unwrap(),
                    Component {
                        name: "Rotation".to_string(),
                        description: "".to_string(),
                        type_: ComponentType::String("Quat".to_string()),
                        attributes: vec![]
                    }
                    .into()
                ),
                (
                    IdentifierPathBuf::new("core::transform::scale").unwrap(),
                    Component {
                        name: "Scale".to_string(),
                        description: "".to_string(),
                        type_: ComponentType::String("Vec3".to_string()),
                        attributes: vec![]
                    }
                    .into()
                ),
                (
                    IdentifierPathBuf::new("core::transform::spherical_billboard").unwrap(),
                    Component {
                        name: "Spherical billboard".to_string(),
                        description: "".to_string(),
                        type_: ComponentType::String("Empty".to_string()),
                        attributes: vec![]
                    }
                    .into()
                ),
                (
                    IdentifierPathBuf::new("core::transform::translation").unwrap(),
                    Component {
                        name: "Translation".to_string(),
                        description: "".to_string(),
                        type_: ComponentType::String("Vec3".to_string()),
                        attributes: vec![]
                    }
                    .into()
                ),
            ]),
            concepts: HashMap::from_iter([
                (
                    IdentifierPathBuf::new("ns").unwrap(),
                    Namespace { name: "Namespace".to_string(), description: "A Test Namespace".to_string() }.into()
                ),
                (
                    IdentifierPathBuf::new("ns::transformable").unwrap(),
                    Concept {
                        name: "Transformable".to_string(),
                        description: "Can be translated, rotated and scaled.".to_string(),
                        extends: vec![],
                        components: HashMap::from_iter([
                            (
                                IdentifierPathBuf::new("core::transform::translation").unwrap(),
                                Value::Array(vec![Value::Integer(0), Value::Integer(0), Value::Integer(0)])
                            ),
                            (
                                IdentifierPathBuf::new("core::transform::rotation").unwrap(),
                                Value::Array(vec![Value::Integer(0), Value::Integer(0), Value::Integer(0), Value::Integer(1)])
                            ),
                            (
                                IdentifierPathBuf::new("core::transform::scale").unwrap(),
                                Value::Array(vec![Value::Integer(1), Value::Integer(1), Value::Integer(1)])
                            )
                        ])
                    }
                    .into()
                )
            ]),
        })
    )
}

#[test]
fn can_validate_identifiers() {
    use Identifier as I;
    use IdentifierPathBuf as IP;

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

    assert_eq!(V::new_from_str("1"), Ok(V::new(1, 0, 0, None)));
    assert_eq!(V::new_from_str("1.0"), Ok(V::new(1, 0, 0, None)));
    assert_eq!(V::new_from_str("1.0.0"), Ok(V::new(1, 0, 0, None)));
    assert_eq!(V::new_from_str("1.2.3"), Ok(V::new(1, 2, 3, None)));
    assert_eq!(V::new_from_str("1.2.3-rc1"), Ok(V::new(1, 2, 3, Some("rc1".to_string()))));

    assert_eq!(V::new_from_str(""), Err(VersionError::TooFewComponents));
    assert_eq!(V::new_from_str("0.0.0"), Err(VersionError::AllZero));
    assert!(matches!(V::new_from_str("1.2.3patch"), Err(VersionError::InvalidNumber(_))));
    assert_eq!(V::new_from_str("1.2.3.4"), Err(VersionError::TooManyComponents));
}

#[test]
fn can_convert_component_types() {
    use ambient_ecs::PrimitiveComponentType as PCT;
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

#[test]
fn can_roundtrip_serialize_versions() {
    let versions = [
        Version::new(1, 0, 0, None),
        Version::new(1, 0, 0, Some("dev".to_string())),
        Version::new(1, 0, 0, Some("rc1".to_string())),
        Version::new(123, 456, 789, Some("rc1".to_string())),
        Version::new(123, 456, 789, None),
    ];

    for version in versions {
        assert_eq!(version, serde_json::from_str(&serde_json::to_string(&version).unwrap()).unwrap());
    }
}
