use std::{collections::HashMap, fmt::Display, path::PathBuf};

use indexmap::IndexMap;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use thiserror::Error;
use url::Url;

use crate::{
    Component, Concept, Enum, ItemPathBuf, Message, PascalCaseIdentifier, SnakeCaseIdentifier,
};

#[derive(Error, Debug, PartialEq)]
pub enum ManifestParseError {
    #[error("manifest was not valid TOML: {0}")]
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
    pub includes: HashMap<SnakeCaseIdentifier, PathBuf>,
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

#[derive(Deserialize, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Default, Serialize)]
#[serde(transparent)]
pub struct PackageId(pub(crate) String);
impl PackageId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generates a new package ID.
    // TODO: suffix with checksum
    pub fn generate() -> Self {
        const DATA_LENGTH: usize = 12;
        const CHECKSUM_LENGTH: usize = 8;
        const TOTAL_LENGTH: usize = DATA_LENGTH + CHECKSUM_LENGTH;

        let data: [u8; DATA_LENGTH] = rand::random();
        let checksum: [u8; CHECKSUM_LENGTH] = sha2::Sha256::digest(&data)[0..CHECKSUM_LENGTH]
            .try_into()
            .unwrap();

        let mut bytes = [0u8; TOTAL_LENGTH];
        bytes[0..DATA_LENGTH].copy_from_slice(&data);
        bytes[DATA_LENGTH..].copy_from_slice(&checksum);

        Self(data_encoding::BASE32_NOPAD.encode(&bytes).to_lowercase())
    }
}
impl Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<PackageId> for SnakeCaseIdentifier {
    fn from(id: PackageId) -> Self {
        SnakeCaseIdentifier(id.0)
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Package {
    /// The ID can be optional if and only if the package is `ambient_core` or an include.
    #[serde(default)]
    pub id: Option<PackageId>,
    pub name: String,
    pub version: Version,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub ambient_version: Option<VersionReq>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub content: PackageContent,
    #[serde(default = "return_true")]
    pub public: bool,
}
impl Default for Package {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            version: Version::parse("0.0.0").unwrap(),
            description: Default::default(),
            repository: Default::default(),
            ambient_version: Default::default(),
            authors: Default::default(),
            content: Default::default(),
            public: true,
        }
    }
}

fn return_true() -> bool {
    true
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
            Url::parse(&ambient_shared_types::urls::deployment_url(&deployment)).ok()
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
        Build, BuildRust, Component, ComponentType, Components, Concept, ConceptValue,
        ContainerType, Dependency, Enum, Identifier, ItemPathBuf, Manifest, ManifestParseError,
        Package, PackageId, PascalCaseIdentifier, SnakeCaseIdentifier,
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
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("test".to_string())),
                    name: "Test".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
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

        [components]
        cell = { type = "i32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["store"] }

        [concepts.Cell]
        name = "Cell"
        description = "A cell object"
        [concepts.Cell.components.required]
        cell = {}
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("tictactoe".to_string())),
                    name: "Tic Tac Toe".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
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
                    ipb("Cell"),
                    Concept {
                        name: Some("Cell".to_string()),
                        description: Some("A cell object".to_string()),
                        extends: vec![],
                        components: Components {
                            required: IndexMap::from_iter([(ipb("cell"), ConceptValue::default())]),
                            optional: Default::default()
                        }
                    }
                )]),
                messages: Default::default(),
                enums: Default::default(),
                includes: Default::default(),
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
        ambient_version = "0.3.0-nightly-2023-08-31"

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("tictactoe".to_string())),
                    name: "Tic Tac Toe".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ambient_version: Some(
                        semver::VersionReq::parse("0.3.0-nightly-2023-08-31").unwrap()
                    ),
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

        [components]
        "core::transform::rotation" = { type = "quat", name = "Rotation", description = "" }
        "core::transform::scale" = { type = "vec3", name = "Scale", description = "" }
        "core::transform::spherical_billboard" = { type = "empty", name = "Spherical billboard", description = "" }
        "core::transform::translation" = { type = "vec3", name = "Translation", description = "" }

        [concepts."ns::Transformable"]
        name = "Transformable"
        description = "Can be translated, rotated and scaled."

        [concepts."ns::Transformable".components.required]
        # This is intentionally out of order to ensure that order is preserved
        "core::transform::translation" = { suggested = [0, 0, 0] }
        "core::transform::scale" = { suggested = [1, 1, 1] }
        "core::transform::rotation" = { suggested = [0, 0, 0, 1] }

        [concepts."ns::Transformable".components.optional]
        "core::transform::inv_local_to_world" = { description = "If specified, will be automatically updated" }
        "#;

        let manifest = Manifest::parse(TOML).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                package: Package {
                    id: Some(PackageId("my_package".to_string())),
                    name: "My Package".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
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
                    ipb("ns::Transformable"),
                    Concept {
                        name: Some("Transformable".to_string()),
                        description: Some("Can be translated, rotated and scaled.".to_string()),
                        extends: vec![],
                        components: Components {
                            required: IndexMap::from_iter([
                                (
                                    ipb("core::transform::translation"),
                                    ConceptValue {
                                        suggested: Some(Value::Array(vec![
                                            Value::Integer(0),
                                            Value::Integer(0),
                                            Value::Integer(0)
                                        ])),
                                        ..Default::default()
                                    }
                                ),
                                (
                                    ipb("core::transform::scale"),
                                    ConceptValue {
                                        suggested: Some(Value::Array(vec![
                                            Value::Integer(1),
                                            Value::Integer(1),
                                            Value::Integer(1)
                                        ])),
                                        ..Default::default()
                                    }
                                ),
                                (
                                    ipb("core::transform::rotation"),
                                    ConceptValue {
                                        suggested: Some(Value::Array(vec![
                                            Value::Integer(0),
                                            Value::Integer(0),
                                            Value::Integer(0),
                                            Value::Integer(1)
                                        ])),
                                        ..Default::default()
                                    }
                                ),
                            ]),
                            optional: IndexMap::from_iter([(
                                ipb("core::transform::inv_local_to_world"),
                                ConceptValue {
                                    description: Some(
                                        "If specified, will be automatically updated".to_string()
                                    ),
                                    ..Default::default()
                                },
                            )])
                        }
                    }
                )]),
                messages: Default::default(),
                enums: Default::default(),
                includes: Default::default(),
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
                .required
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
                    id: Some(PackageId("tictactoe".to_string())),
                    name: "Tic Tac Toe".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
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
                includes: Default::default(),
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

        [components]
        test = { type = "I32", name = "Test", description = "Test" }
        vec_test = { type = { container_type = "Vec", element_type = "I32" }, name = "Test", description = "Test" }
        option_test = { type = { container_type = "Option", element_type = "I32" }, name = "Test", description = "Test" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("test".to_string())),
                    name: "Test".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
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
                includes: Default::default(),
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
                    id: Some(PackageId("dependencies".to_string())),
                    name: "dependencies".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                build: Default::default(),
                components: Default::default(),
                concepts: Default::default(),
                messages: Default::default(),
                enums: Default::default(),
                includes: Default::default(),
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
