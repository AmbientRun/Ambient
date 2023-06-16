use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Component, Concept, Identifier, IdentifierPathBuf, Message, Version};
use anyhow::Context;

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Serialize)]
pub struct Manifest {
    #[serde(default)]
    #[serde(alias = "project")]
    pub ember: Ember,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub components: BTreeMap<IdentifierPathBuf, NamespaceOr<Component>>,
    #[serde(default)]
    pub concepts: BTreeMap<IdentifierPathBuf, NamespaceOr<Concept>>,
    #[serde(default)]
    pub messages: BTreeMap<IdentifierPathBuf, NamespaceOr<Message>>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, toml::de::Error> {
        let parsed_manifest = toml::from_str(manifest)?;
        let raw = toml::from_str::<toml::Table>(manifest)?;
        if raw.contains_key("project") {
            log::warn!("The `project` key is deprecated. Please use `ember` instead.");
        }
        Ok(parsed_manifest)
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
        self.ember
            .organization
            .iter()
            .chain(std::iter::once(&self.ember.id))
            .cloned()
            .collect()
    }

    fn resolve_imports(&mut self, directory: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut new_includes = vec![];
        for include in &self.ember.includes {
            let manifest = Manifest::from_file(directory.as_ref().join(include))?;
            new_includes.extend(manifest.ember.includes);
            self.components.extend(manifest.components);
            self.concepts.extend(manifest.concepts);
            self.messages.extend(manifest.messages);
        }
        self.ember.includes.extend(new_includes);
        Ok(())
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Ember {
    pub id: Identifier,
    pub name: Option<String>,
    pub version: Version,
    pub description: Option<String>,
    pub repository: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default, rename = "type")]
    pub type_: EmberType,
    #[serde(default)]
    pub categories: Vec<Category>,
    pub organization: Option<Identifier>,
    #[serde(default)]
    pub includes: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize, Default)]
pub enum EmberType {
    #[default]
    Game,
    Mod,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub enum Category {
    Example,
    Fps,
    Survival,
    Simulation,
    Multiplayer,
    Strategy,
    Sports,
    Racing,
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

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
pub struct Namespace {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum NamespaceOr<T> {
    Other(T),
    Namespace(Namespace),
}
impl<T> NamespaceOr<T> {
    pub fn other(&self) -> Option<&T> {
        match self {
            NamespaceOr::Other(o) => Some(o),
            NamespaceOr::Namespace(_) => None,
        }
    }

    pub fn namespace(&self) -> Option<&Namespace> {
        match self {
            NamespaceOr::Other(_) => None,
            NamespaceOr::Namespace(n) => Some(n),
        }
    }
}

impl<T> From<Namespace> for NamespaceOr<T> {
    fn from(value: Namespace) -> Self {
        Self::Namespace(value)
    }
}
impl From<Component> for NamespaceOr<Component> {
    fn from(value: Component) -> Self {
        Self::Other(value)
    }
}
impl From<Concept> for NamespaceOr<Concept> {
    fn from(value: Concept) -> Self {
        Self::Other(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        Build, BuildRust, Component, ComponentType, Concept, Ember, Identifier, IdentifierPathBuf,
        Manifest, Namespace, Version, VersionSuffix,
    };

    #[test]
    fn can_parse_minimal_toml() {
        const TOML: &str = r#"
        [ember]
        id = "test"
        name = "Test"
        version = "0.0.1"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: Identifier::new("test").unwrap(),
                    name: Some("Test".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    ..Default::default()
                },
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_minimal_legacy_toml() {
        const TOML: &str = r#"
        [project]
        id = "test"
        name = "Test"
        version = "0.0.1"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: Identifier::new("test").unwrap(),
                    name: Some("Test".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    ..Default::default()
                },
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_tictactoe_toml() {
        const TOML: &str = r#"
        [ember]
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
                ember: Ember {
                    id: Identifier::new("tictactoe").unwrap(),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    ..Default::default()
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
                        type_: ComponentType::String("I32".to_string()),
                        attributes: vec!["Store".to_string()],
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
            })
        )
    }

    #[test]
    fn can_parse_rust_build_settings() {
        const TOML: &str = r#"
        [ember]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: Identifier::new("tictactoe").unwrap(),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_manifest_with_namespaces() {
        const TOML: &str = r#"
        [ember]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"

        [components]
        "core" = { name = "Core" }
        "core::app" = { name = "App" }

        "core::app::main_scene" = { name = "Main Scene", type = "Empty" }
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                ember: Ember {
                    id: Identifier::new("tictactoe").unwrap(),
                    name: Some("Tic Tac Toe".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: BTreeMap::from_iter([
                    (
                        IdentifierPathBuf::new("core").unwrap(),
                        Namespace {
                            name: Some("Core".to_string()),
                            description: None
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::app").unwrap(),
                        Namespace {
                            name: Some("App".to_string()),
                            description: None
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::app::main_scene").unwrap(),
                        Component {
                            name: Some("Main Scene".to_string()),
                            description: None,
                            type_: ComponentType::String("Empty".to_string()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    )
                ]),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
            })
        )
    }

    #[test]
    fn can_parse_concepts_with_documented_namespace_from_manifest() {
        use toml::Value;

        const TOML: &str = r#"
        [ember]
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
                ember: Ember {
                    id: Identifier::new("my_project").unwrap(),
                    name: Some("My Project".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    ..Default::default()
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
                            type_: ComponentType::String("Quat".to_string()),
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
                            type_: ComponentType::String("Vec3".to_string()),
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
                            type_: ComponentType::String("Empty".to_string()),
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
                            type_: ComponentType::String("Vec3".to_string()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                ]),
                concepts: BTreeMap::from_iter([
                    (
                        IdentifierPathBuf::new("ns").unwrap(),
                        Namespace {
                            name: Some("Namespace".to_string()),
                            description: Some("A Test Namespace".to_string())
                        }
                        .into()
                    ),
                    (
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
                    )
                ]),
                messages: BTreeMap::new(),
            })
        )
    }
}
