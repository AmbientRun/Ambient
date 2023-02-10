use super::implementation;

#[test]
fn can_generate_components_from_manifest_in_global_namespace() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "runtime_components"
        name = "Runtime Components"

        [components]
        "core::app::main_scene" = { name = "Main Scene", description = "", type = "Empty" }
        "core::app::name" = { name = "name", description = "", type = "String" }
        "core::camera::active_camera" = { name = "Active Camera", description = "No description provided", type = "F32" }
        "core::camera::aspect_ratio" = { name = "Aspect Ratio", description = "", type = "F32" }
        "core::rendering::joints" = { name = "Joints", description = "No description provided", type = { type = "Vec", element_type = "EntityId" } }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
        #[allow(missing_docs)]
        pub mod components {
            pub mod core {
                pub mod app {
                    static MAIN_SCENE: crate::LazyComponent<()> = crate::lazy_component!("core::app::main_scene");
                    #[doc = "**Main Scene**"]
                    pub fn main_scene() -> crate::Component<()> {
                        *MAIN_SCENE
                    }
                    static NAME: crate::LazyComponent<String> = crate::lazy_component!("core::app::name");
                    #[doc = "**name**"]
                    pub fn name() -> crate::Component<String> {
                        *NAME
                    }
                }
                pub mod camera {
                    static ACTIVE_CAMERA: crate::LazyComponent<f32> = crate::lazy_component!("core::camera::active_camera");
                    #[doc = "**Active Camera**\n\nNo description provided"]
                    pub fn active_camera() -> crate::Component<f32> {
                        *ACTIVE_CAMERA
                    }
                    static ASPECT_RATIO: crate::LazyComponent<f32> = crate::lazy_component!("core::camera::aspect_ratio");
                    #[doc = "**Aspect Ratio**"]
                    pub fn aspect_ratio() -> crate::Component<f32> {
                        *ASPECT_RATIO
                    }
                }
                pub mod rendering {
                    static JOINTS: crate::LazyComponent< Vec<crate::EntityId> > = crate::lazy_component!("core::rendering::joints");
                    #[doc = "**Joints**\n\nNo description provided"]
                    pub fn joints() -> crate::Component< Vec<crate::EntityId> > {
                        *JOINTS
                    }
                }
            }
        }
    };

    let result = implementation(
        ("elementsy.toml".to_string(), manifest.to_string()),
        &[],
        true,
    )
    .unwrap();
    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_extend_existing_components_in_global_namespace() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "runtime_components"
        name = "Runtime Components"

        [components]
        "core::app::main_scene" = { name = "Main Scene", description = "", type = "Empty" }
        "core::camera::active_camera" = { name = "Active Camera", description = "No description provided", type = "F32" }
        "core::rendering::joints" = { name = "Joints", description = "No description provided", type = { type = "Vec", element_type = "EntityId" } }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
        #[allow(missing_docs)]
        pub mod components {
            pub mod core {
                pub mod app {
                    pub use base::components::core::app::*;
                    static MAIN_SCENE: crate::LazyComponent<()> = crate::lazy_component!("core::app::main_scene");
                    #[doc = "**Main Scene**"]
                    pub fn main_scene() -> crate::Component<()> {
                        *MAIN_SCENE
                    }
                }
                pub mod camera {
                    pub use base::components::core::camera::*;
                    static ACTIVE_CAMERA: crate::LazyComponent<f32> = crate::lazy_component!("core::camera::active_camera");
                    #[doc = "**Active Camera**\n\nNo description provided"]
                    pub fn active_camera() -> crate::Component<f32> {
                        *ACTIVE_CAMERA
                    }
                }
                pub mod player {
                    pub use base::components::core::player::*;
                }
                pub mod rendering {
                    static JOINTS: crate::LazyComponent< Vec<crate::EntityId> > = crate::lazy_component!("core::rendering::joints");
                    #[doc = "**Joints**\n\nNo description provided"]
                    pub fn joints() -> crate::Component< Vec<crate::EntityId> > {
                        *JOINTS
                    }
                }
            }
        }
    };

    let result = implementation(
        ("elementsy.toml".to_string(), manifest.to_string()),
        &[
            vec![
                "base".to_string(),
                "components".to_string(),
                "core".to_string(),
                "app".to_string(),
            ],
            vec![
                "base".to_string(),
                "components".to_string(),
                "core".to_string(),
                "camera".to_string(),
            ],
            vec![
                "base".to_string(),
                "components".to_string(),
                "core".to_string(),
                "player".to_string(),
            ],
        ],
        true,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_accept_no_components() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
        #[allow(missing_docs)]
        pub mod components {}
    };

    let result = implementation(
        ("elementsy.toml".to_string(), manifest.to_string()),
        &[],
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_generate_components_from_manifest() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        a_cool_component = { name = "Cool Component", description = "", type = "Empty" }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
        #[allow(missing_docs)]
        pub mod components {
            static A_COOL_COMPONENT: crate::LazyComponent<()> = crate::lazy_component!("my_project::a_cool_component");
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> crate::Component<()> {
                *A_COOL_COMPONENT
            }
        }
    };

    let result = implementation(
        ("elementsy.toml".to_string(), manifest.to_string()),
        &[],
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_generate_components_from_manifest_with_org() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"
        organization = "evil_corp"

        [components]
        a_cool_component = { name = "Cool Component", description = "", type = "Empty" }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
        #[allow(missing_docs)]
        pub mod components {
            static A_COOL_COMPONENT: crate::LazyComponent<()> = crate::lazy_component!("evil_corp::my_project::a_cool_component");
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> crate::Component<()> {
                *A_COOL_COMPONENT
            }
        }
    };

    let result = implementation(
        ("elementsy.toml".to_string(), manifest.to_string()),
        &[],
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}
