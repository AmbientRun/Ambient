use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{
    Component, Concept, Enum, ItemPathBuf, Message, PascalCaseIdentifier, SnakeCaseIdentifier,
};
use semver::{Version, VersionReq};

#[derive(Error, Debug, PartialEq)]
pub enum ManifestParseError {
    #[error("manifest was not valid TOML")]
    TomlError(#[from] toml::de::Error),
    #[error("manifest contains a project and/or an ember section; projects/embers have been renamed to packages")]
    ProjectEmberRenamedToPackageError,
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Serialize)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    #[serde(alias = "component")]
    pub components: IndexMap<ItemPathBuf, Component>,
    #[serde(default)]
    #[serde(alias = "concept")]
    pub concepts: IndexMap<ItemPathBuf, Concept>,
    #[serde(default)]
    #[serde(alias = "message")]
    pub messages: IndexMap<ItemPathBuf, Message>,
    #[serde(default)]
    #[serde(alias = "enum")]
    pub enums: IndexMap<PascalCaseIdentifier, Enum>,
    #[serde(default)]
    pub dependencies: IndexMap<SnakeCaseIdentifier, Dependency>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, ManifestParseError> {
        let raw = toml::from_str::<toml::Table>(manifest)?;
        if raw.contains_key("project") || raw.contains_key("ember") {
            return Err(ManifestParseError::ProjectEmberRenamedToPackageError);
        }

        Ok(toml::from_str(manifest)?)
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string_pretty(self).unwrap()
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Package {
    pub id: SnakeCaseIdentifier,
    pub name: String,
    pub version: Option<Version>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub ambient_version: Option<AmbientRuntimeVersion>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub content: PackageContent,
    #[serde(default = "return_true")]
    pub public: bool,
    #[serde(default)]
    pub includes: Vec<PathBuf>,
}

fn return_true() -> bool {
    true
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    parse_display::Display,
    parse_display::FromStr,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
)]
pub enum AmbientRuntimeVersion {
    #[display("{0}")]
    Stable(VersionReq),
    #[display("nightly-{date}")]
    Nightly { date: String },
}
#[test]
fn test_ambient_runtime_version() {
    assert_eq!(
        "1.2.3".parse(),
        Ok(AmbientRuntimeVersion::Stable(
            VersionReq::parse("1.2.3").unwrap()
        ))
    );
    assert_eq!(
        "nightly-2021-01-01".parse(),
        Ok(AmbientRuntimeVersion::Nightly {
            date: "2021-01-01".to_string()
        })
    );
    assert_eq!(
        AmbientRuntimeVersion::Stable(VersionReq::parse("1.2.3").unwrap()).to_string(),
        "^1.2.3"
    );
    assert_eq!(
        AmbientRuntimeVersion::Nightly {
            date: "2021-01-01".to_string()
        }
        .to_string(),
        "nightly-2021-01-01"
    );
}

// ----- NOTE: Update docs/reference/package.md when changing this ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PackageContent {
    Playable {
        #[serde(default)]
        example: bool,
    },
    /// Assets are something that you can use as a dependency in your package
    Asset {
        #[serde(default)]
        models: bool,
        #[serde(default)]
        animations: bool,
        #[serde(default)]
        textures: bool,
        #[serde(default)]
        materials: bool,
        #[serde(default)]
        audio: bool,
        #[serde(default)]
        fonts: bool,
        #[serde(default)]
        code: bool,
        #[serde(default)]
        schema: bool,
    },
    Tool,
    Mod {
        /// List of package ids that this mod is applicable to
        #[serde(default)]
        for_playables: Vec<String>,
    },
}
impl Default for PackageContent {
    fn default() -> Self {
        Self::Playable { example: false }
    }
}

// -----------------------------------------------------------------

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

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Dependency {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    url: Option<Url>,
    #[serde(default)]
    deployment: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}
impl Dependency {
    pub fn url(&self) -> Option<Url> {
        if let Some(url) = self.url.clone() {
            Some(url)
        } else if let Some(deployment) = self.deployment.as_ref() {
            Url::parse(&format!("https://assets.ambient.run/{deployment}")).ok()
        } else {
            None
        }
    }

    pub fn has_remote_dependency(&self) -> bool {
        self.url().is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use indexmap::IndexMap;
    use url::Url;

    use crate::{
        Build, BuildRust, Component, ComponentType, Concept, ContainerType, Dependency, Enum,
        Identifier, ItemPathBuf, Manifest, ManifestParseError, Package, PascalCaseIdentifier,
        SnakeCaseIdentifier,
    };
    use semver::Version;

    fn i(s: &str) -> Identifier {
        Identifier::new(s).unwrap()
    }

    fn sci(s: &str) -> SnakeCaseIdentifier {
        SnakeCaseIdentifier::new(s).unwrap()
    }

    fn pci(s: &str) -> PascalCaseIdentifier {
        PascalCaseIdentifier::new(s).unwrap()
    }

    fn ipb(s: &str) -> ItemPathBuf {
        ItemPathBuf::new(s).unwrap()
    }

    #[test]
    fn can_parse_minimal_toml() {
        const TOML: &str = r#"
        [package]
        id = "test"
        name = "Test"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: SnakeCaseIdentifier::new("test").unwrap(),
                    name: "Test".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                ..Default::default()
            })
        );
    }

    #[test]
    fn will_fail_on_legacy_project_toml() {
        const TOML: &str = r#"
        [project]
        id = "test"
        name = "Test"
        version = "0.0.1"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Err(ManifestParseError::ProjectEmberRenamedToPackageError)
        )
    }

    #[test]
    fn can_parse_tictactoe_toml() {
        const TOML: &str = r#"
        [package]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false

        [components]
        cell = { type = "i32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["store"] }

        [concepts.cell]
        name = "Cell"
        description = "A cell object"
        [concepts.cell.components]
        cell = 0
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: sci("tictactoe"),
                    name: "Tic Tac Toe".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([(
                    ipb("cell"),
                    Component {
                        name: Some("Cell".to_string()),
                        description: Some("The ID of the cell this player is in".to_string()),
                        type_: ComponentType::Item(i("i32").into()),
                        attributes: vec![i("store").into()],
                        default: None,
                    }
                )]),
                concepts: IndexMap::from_iter([(
                    ipb("cell"),
                    Concept {
                        name: Some("Cell".to_string()),
                        description: Some("A cell object".to_string()),
                        extends: vec![],
                        components: IndexMap::from_iter([(ipb("cell"), toml::Value::Integer(0))])
                    }
                )]),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: Default::default(),
            })
        )
    }

    #[test]
    fn can_parse_rust_build_settings() {
        const TOML: &str = r#"
        [package]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: sci("tictactoe"),
                    name: "Tic Tac Toe".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_concepts_with_documented_namespace_from_manifest() {
        use toml::Value;

        const TOML: &str = r#"
        [package]
        id = "my_package"
        name = "My Package"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false

        [components]
        "core::transform::rotation" = { type = "quat", name = "Rotation", description = "" }
        "core::transform::scale" = { type = "vec3", name = "Scale", description = "" }
        "core::transform::spherical_billboard" = { type = "empty", name = "Spherical billboard", description = "" }
        "core::transform::translation" = { type = "vec3", name = "Translation", description = "" }

        [concepts."ns::transformable"]
        name = "Transformable"
        description = "Can be translated, rotated and scaled."

        [concepts."ns::transformable".components]
        # This is intentionally out of order to ensure that order is preserved
        "core::transform::translation" = [0, 0, 0]
        "core::transform::scale" = [1, 1, 1]
        "core::transform::rotation" = [0, 0, 0, 1]
        "#;

        let manifest = Manifest::parse(TOML).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                package: Package {
                    id: sci("my_package"),
                    name: "My Package".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ipb("core::transform::rotation"),
                        Component {
                            name: Some("Rotation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("quat").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("core::transform::scale"),
                        Component {
                            name: Some("Scale".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("core::transform::spherical_billboard"),
                        Component {
                            name: Some("Spherical billboard".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("empty").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("core::transform::translation"),
                        Component {
                            name: Some("Translation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                ]),
                concepts: IndexMap::from_iter([(
                    ipb("ns::transformable"),
                    Concept {
                        name: Some("Transformable".to_string()),
                        description: Some("Can be translated, rotated and scaled.".to_string()),
                        extends: vec![],
                        components: IndexMap::from_iter([
                            (
                                ipb("core::transform::translation"),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0)
                                ])
                            ),
                            (
                                ipb("core::transform::scale"),
                                Value::Array(vec![
                                    Value::Integer(1),
                                    Value::Integer(1),
                                    Value::Integer(1)
                                ])
                            ),
                            (
                                ipb("core::transform::rotation"),
                                Value::Array(vec![
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(0),
                                    Value::Integer(1)
                                ])
                            ),
                        ])
                    }
                )]),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: Default::default(),
            }
        );

        assert_eq!(
            manifest
                .concepts
                .first()
                .unwrap()
                .1
                .components
                .keys()
                .collect::<Vec<_>>(),
            vec![
                &ipb("core::transform::translation"),
                &ipb("core::transform::scale"),
                &ipb("core::transform::rotation"),
            ]
        );
    }

    #[test]
    fn can_parse_enums() {
        const TOML: &str = r#"
        [package]
        id = "tictactoe"
        name = "Tic Tac Toe"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false

        [enums.CellState]
        description = "The current cell state"
        [enums.CellState.members]
        Taken = "The cell is taken"
        Free = "The cell is free"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: sci("tictactoe"),
                    name: "Tic Tac Toe".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                build: Build::default(),
                components: Default::default(),
                concepts: Default::default(),
                messages: Default::default(),
                enums: IndexMap::from_iter([(
                    pci("CellState"),
                    Enum {
                        description: Some("The current cell state".to_string()),
                        members: IndexMap::from_iter([
                            (pci("Taken"), "The cell is taken".to_string()),
                            (pci("Free"), "The cell is free".to_string()),
                        ])
                    }
                )]),
                dependencies: Default::default(),
            })
        )
    }

    #[test]
    fn can_parse_container_types() {
        const TOML: &str = r#"
        [package]
        id = "test"
        name = "Test"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false

        [components]
        test = { type = "I32", name = "Test", description = "Test" }
        vec_test = { type = { container_type = "Vec", element_type = "I32" }, name = "Test", description = "Test" }
        option_test = { type = { container_type = "Option", element_type = "I32" }, name = "Test", description = "Test" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: sci("test"),
                    name: "Test".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ipb("test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Item(i("I32").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("vec_test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Vec,
                                element_type: i("I32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("option_test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Option,
                                element_type: i("I32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                    )
                ]),
                concepts: Default::default(),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: Default::default(),
            })
        )
    }

    #[test]
    fn can_parse_dependencies() {
        const TOML: &str = r#"
        [package]
        id = "dependencies"
        name = "dependencies"
        version = "0.0.1"
        content = { type = "Playable" }
        public = false

        [dependencies]
        deps_assets = { path = "deps/assets" }
        deps_code = { path = "deps/code" }
        deps_ignore_me = { path = "deps/ignore_me", enabled = false }
        deps_remote = { url = "http://example.com", enabled = true }
        deps_remote_deployment = { deployment = "jhsdfu574S" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: sci("dependencies"),
                    name: "dependencies".to_string(),
                    version: Some(Version::parse("0.0.1").unwrap()),
                    ..Default::default()
                },
                build: Default::default(),
                components: Default::default(),
                concepts: Default::default(),
                messages: Default::default(),
                enums: Default::default(),
                dependencies: IndexMap::from_iter([
                    (
                        sci("deps_assets"),
                        Dependency {
                            path: Some(PathBuf::from("deps/assets")),
                            url: None,
                            deployment: None,
                            enabled: None,
                        }
                    ),
                    (
                        sci("deps_code"),
                        Dependency {
                            path: Some(PathBuf::from("deps/code")),
                            url: None,
                            deployment: None,
                            enabled: None,
                        }
                    ),
                    (
                        sci("deps_ignore_me"),
                        Dependency {
                            path: Some(PathBuf::from("deps/ignore_me")),
                            url: None,
                            deployment: None,
                            enabled: Some(false),
                        }
                    ),
                    (
                        sci("deps_remote"),
                        Dependency {
                            path: None,
                            url: Some(Url::parse("http://example.com").unwrap()),
                            deployment: None,
                            enabled: Some(true),
                        }
                    ),
                    (
                        sci("deps_remote_deployment"),
                        Dependency {
                            path: None,
                            url: None,
                            deployment: Some("jhsdfu574S".to_owned()),
                            enabled: None,
                        }
                    )
                ])
            })
        )
    }
}
