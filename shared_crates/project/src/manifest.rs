use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    CollisionResponse, Component, Concept, Identifier, IdentifierPathBuf, Layer, Message, Version,
};

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub physics: Physics,
    #[serde(default)]
    pub components: BTreeMap<IdentifierPathBuf, NamespaceOr<Component>>,
    #[serde(default)]
    pub concepts: BTreeMap<IdentifierPathBuf, NamespaceOr<Concept>>,
    #[serde(default)]
    pub messages: BTreeMap<IdentifierPathBuf, NamespaceOr<Message>>,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(manifest)
    }

    pub fn project_path(&self) -> IdentifierPathBuf {
        self.project
            .organization
            .iter()
            .chain(std::iter::once(&self.project.id))
            .cloned()
            .collect()
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Project {
    pub id: Identifier,
    pub name: Option<String>,
    pub version: Version,
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub organization: Option<Identifier>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Build {
    #[serde(default)]
    pub rust: BuildRust,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Physics {
    #[serde(default)]
    pub layers: BTreeMap<IdentifierPathBuf, Layer>,
    #[serde(default)]
    pub collision_responses: Vec<CollisionResponse>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Namespace {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
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
        Build, BuildRust, CollisionFilter, CollisionResponse, Component, ComponentType, Concept,
        Identifier, IdentifierPathBuf, Layer, Manifest, Namespace, Physics, Project, Version,
        VersionSuffix,
    };

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
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    description: None,
                    authors: vec![],
                    organization: None
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                physics: Default::default(),
                components: BTreeMap::from_iter([(
                    IdentifierPathBuf::new("cell").unwrap(),
                    Component {
                        name: "Cell".to_string(),
                        description: "The ID of the cell this player is in".to_string(),
                        type_: ComponentType::String("I32".to_string()),
                        attributes: vec!["Store".to_string()],
                        default: None,
                    }
                    .into()
                )]),
                concepts: BTreeMap::from_iter([(
                    IdentifierPathBuf::new("cell").unwrap(),
                    Concept {
                        name: "Cell".to_string(),
                        description: "A cell object".to_string(),
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
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    description: None,
                    authors: vec![],
                    organization: None
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                physics: Default::default(),
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
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
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    description: None,
                    authors: vec![],
                    organization: None
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                physics: Default::default(),
                components: BTreeMap::from_iter([
                    (
                        IdentifierPathBuf::new("core").unwrap(),
                        Namespace {
                            name: "Core".to_string(),
                            description: String::new()
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::app").unwrap(),
                        Namespace {
                            name: "App".to_string(),
                            description: String::new()
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::app::main_scene").unwrap(),
                        Component {
                            name: "Main Scene".to_string(),
                            description: "".to_string(),
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
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    description: None,
                    authors: vec![],
                    organization: None
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                physics: Default::default(),
                components: BTreeMap::from_iter([
                    (
                        IdentifierPathBuf::new("core::transform::rotation").unwrap(),
                        Component {
                            name: "Rotation".to_string(),
                            description: "".to_string(),
                            type_: ComponentType::String("Quat".to_string()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::transform::scale").unwrap(),
                        Component {
                            name: "Scale".to_string(),
                            description: "".to_string(),
                            type_: ComponentType::String("Vec3".to_string()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::transform::spherical_billboard").unwrap(),
                        Component {
                            name: "Spherical billboard".to_string(),
                            description: "".to_string(),
                            type_: ComponentType::String("Empty".to_string()),
                            attributes: vec![],
                            default: None,
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("core::transform::translation").unwrap(),
                        Component {
                            name: "Translation".to_string(),
                            description: "".to_string(),
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
                            name: "Namespace".to_string(),
                            description: "A Test Namespace".to_string()
                        }
                        .into()
                    ),
                    (
                        IdentifierPathBuf::new("ns::transformable").unwrap(),
                        Concept {
                            name: "Transformable".to_string(),
                            description: "Can be translated, rotated and scaled.".to_string(),
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

    #[test]
    fn can_parse_physics_settings() {
        const TOML: &str = r#"
        [project]
        id = "physics_example"
        name = "Physics Example"
        version = "0.0.1"

        [physics]
        collision_responses = [
            { layer_a = "player", layer_b = "projectile", filter = "Block" },
            { layer_a = "player", layer_b = "pickup", filter = "Overlap" },
            { layer_a = "pickup", layer_b = "projectile", filter = "Ignore" }
        ]

        [physics.layers.player]
        name = "Player"
        description = "Layer for player collisions"

        [physics.layers.projectile]
        name = "Projectile"
        description = "Layer for projectiles"

        [physics.layers.pickup]
        name = "Pick-Up"
        description = "Layer for pick-ups"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                project: Project {
                    id: Identifier::new("physics_example").unwrap(),
                    name: Some("Physics Example".to_string()),
                    version: Version::new(0, 0, 1, VersionSuffix::Final),
                    description: None,
                    authors: vec![],
                    organization: None
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                physics: Physics {
                    layers: BTreeMap::from_iter([
                        (
                            IdentifierPathBuf::new("player").unwrap(),
                            Layer {
                                name: "Player".to_string(),
                                description: "Layer for player collisions".to_string(),
                            }
                        ),
                        (
                            IdentifierPathBuf::new("projectile").unwrap(),
                            Layer {
                                name: "Projectile".to_string(),
                                description: "Layer for projectiles".to_string(),
                            }
                        ),
                        (
                            IdentifierPathBuf::new("pickup").unwrap(),
                            Layer {
                                name: "Pick-Up".to_string(),
                                description: "Layer for pick-ups".to_string(),
                            }
                        )
                    ]),
                    collision_responses: vec![
                        CollisionResponse {
                            layer_a: IdentifierPathBuf::new("player").unwrap(),
                            layer_b: IdentifierPathBuf::new("projectile").unwrap(),
                            filter: CollisionFilter::Block
                        },
                        CollisionResponse {
                            layer_a: IdentifierPathBuf::new("player").unwrap(),
                            layer_b: IdentifierPathBuf::new("pickup").unwrap(),
                            filter: CollisionFilter::Overlap
                        },
                        CollisionResponse {
                            layer_a: IdentifierPathBuf::new("pickup").unwrap(),
                            layer_b: IdentifierPathBuf::new("projectile").unwrap(),
                            filter: CollisionFilter::Ignore
                        }
                    ]
                },
                components: BTreeMap::new(),
                concepts: BTreeMap::new(),
                messages: BTreeMap::new(),
            })
        )
    }
}
