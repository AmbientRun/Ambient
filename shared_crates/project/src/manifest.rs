use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    CamelCaseIdentifier, Component, Concept, Enum, Identifier, IdentifierPathBuf, Message, Version,
};
use anyhow::Context;

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub components: BTreeMap<IdentifierPathBuf, Component>,
    #[serde(default)]
    pub concepts: BTreeMap<IdentifierPathBuf, Concept>,
    #[serde(default)]
    pub messages: BTreeMap<IdentifierPathBuf, Message>,
    #[serde(default)]
    pub enums: BTreeMap<CamelCaseIdentifier, Enum>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(manifest)
    }
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut res = Self::parse(
            &fs::read_to_string(path.as_ref())
                .context(format!("Failed to read file: {:?}", path.as_ref()))?,
        )?;
        res.resolve_imports(path.as_ref().parent().context("No parent directory")?)?;
        Ok(res)
    }

    pub fn project_path(&self) -> IdentifierPathBuf {
        self.project
            .organization
            .iter()
            .chain(std::iter::once(&self.project.id))
            .cloned()
            .collect()
    }

    fn resolve_imports(&mut self, directory: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut new_includes = vec![];
        for include in &self.project.includes {
            let manifest = Manifest::from_file(directory.as_ref().join(include))?;
            new_includes.extend(manifest.project.includes);
            self.components.extend(manifest.components);
            self.concepts.extend(manifest.concepts);
            self.messages.extend(manifest.messages);
        }
        self.project.includes.extend(new_includes);
        Ok(())
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Project {
    pub id: Identifier,
    pub name: Option<String>,
    pub version: Option<Version>,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<Identifier>,
    #[serde(default)]
    pub includes: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Build {
    #[serde(default)]
    pub rust: BuildRust,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct BuildRust {
    #[serde(rename = "feature-multibuild")]
    pub feature_multibuild: Vec<String>,
}
impl Default for BuildRust {
    fn default() -> Self {
        Self {
            feature_multibuild: vec!["client".to_string(), "server".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        Build, BuildRust, CamelCaseIdentifier, Component, ComponentType, Concept, ContainerType,
        Enum, EnumMember, Identifier, IdentifierPathBuf, Manifest, Project, Version, VersionSuffix,
    };

    fn cci(s: &str) -> CamelCaseIdentifier {
        CamelCaseIdentifier::new(s).unwrap()
    }

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
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: BTreeMap::from_iter([(
                    IdentifierPathBuf::new("cell").unwrap(),
                    Component {
                        name: Some("Cell".to_string()),
                        description: Some("The ID of the cell this player is in".to_string()),
                        type_: ComponentType::Identifier(cci("I32")),
                        attributes: vec![cci("Store")],
                        default: None,
                    }
                    .into()
                )]),
                concepts: BTreeMap::from_iter([(
                    IdentifierPathBuf::new("cell").unwrap(),
                    Concept {
                        name: Some("Cell".to_string()),
                        description: Some("A cell object".to_string()),
                        extends: vec![],
                        components: BTreeMap::from_iter([(
                            IdentifierPathBuf::new("cell").unwrap(),
                            toml::Value::Integer(0)
                        )])
                    }
                    .into()
                )]),
                messages: BTreeMap::new(),
                enums: BTreeMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_rust_build_settings() {
        const TOML: &str = r#"
        [project]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: Identifier::new("tictactoe").unwrap(),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
                enums: BTreeMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_concepts_with_documented_namespace_from_manifest() {
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
        "ns::transformable" = { name = "Transformable", description = "Can be translated, rotated and scaled.", components = {"core::transform::translation" = [0, 0, 0], "core::transform::rotation" = [0, 0, 0, 1], "core::transform::scale" = [1, 1, 1]} }
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: Identifier::new("my_project").unwrap(),
                    name: Some("My Project".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: BTreeMap::from_iter([
                    (
                        IdentifierPathBuf::new("core::transform::rotation").unwrap(),
                        Component {
                            name: Some("Rotation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Identifier(cci("Quat")),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::transform::scale").unwrap(),
                        Component {
                            name: Some("Scale".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Identifier(cci("Vec3")),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::transform::spherical_billboard").unwrap(),
                        Component {
                            name: Some("Spherical billboard".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Identifier(cci("Empty")),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::transform::translation").unwrap(),
                        Component {
                            name: Some("Translation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Identifier(cci("Vec3")),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                ]),
                concepts: BTreeMap::from_iter([(
                    IdentifierPathBuf::new("ns::transformable").unwrap(),
                    Concept {
                        name: Some("Transformable".to_string()),
                        description: Some("Can be translated, rotated and scaled.".to_string()),
                        extends: vec![],
                        components: BTreeMap::from_iter([
                            (
                                IdentifierPathBuf::new("core::transform::translation").unwrap(),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0)
                                ])
                            ),
                            (
                                IdentifierPathBuf::new("core::transform::rotation").unwrap(),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(1)
                                ])
                            ),
                            (
                                IdentifierPathBuf::new("core::transform::scale").unwrap(),
                                Value::Array(vec![
                                    Value::Integer(1),
                                    Value::Integer(1),
                                    Value::Integer(1)
                                ])
                            )
                        ])
                    }
                    .into()
                )]),
                messages: BTreeMap::new(),
                enums: BTreeMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_enums() {
        const TOML: &str = r#"
        [project]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [enums]
        CellState = [
            { name = "Free", description = "The cell is free" },
            { name = "Taken", description = "The cell is taken" }
        ]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: Identifier::new("tictactoe").unwrap(),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build::default(),
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
                enums: BTreeMap::from_iter([(
                    CamelCaseIdentifier::new("CellState").unwrap(),
                    Enum(vec![
                        EnumMember {
                            name: cci("Free"),
                            description: Some("The cell is free".to_string()),
                        },
                        EnumMember {
                            name: cci("Taken"),
                            description: Some("The cell is taken".to_string()),
                        }
                    ])
                    .into()
                )]),
            })
        )
    }

    #[test]
    fn can_parse_container_types() {
        const TOML: &str = r#"
        [project]
        id = "test"
        name = "Test"
        version = "0.0.1"

        [components]
        test = { type = "I32", name = "Test", description = "Test" }
        vec_test = { type = { container_type = "Vec", element_type = "I32" }, name = "Test", description = "Test" }
        option_test = { type = { container_type = "Option", element_type = "I32" }, name = "Test", description = "Test" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: Identifier::new("test").unwrap(),
                    name: Some("Test".to_string()),
                    version: Some(Version::new(0, 0, 1, VersionSuffix::Final)),
                    description: None,
                    authors: vec![],
                    organization: None,
                    includes: Default::default(),
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: BTreeMap::from_iter([
                    (
                        IdentifierPathBuf::new("test").unwrap(),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Identifier(cci("I32")),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("vec_test").unwrap(),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::ContainerType {
                                type_: ContainerType::Vec,
                                element_type: cci("I32")
                            },
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("option_test").unwrap(),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::ContainerType {
                                type_: ContainerType::Option,
                                element_type: cci("I32")
                            },
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    )
                ]),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
                enums: BTreeMap::new(),
            })
        )
    }
}
