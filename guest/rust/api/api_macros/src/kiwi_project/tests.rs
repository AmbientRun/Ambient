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
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[allow(missing_docs)]
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the corresponding project."]
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
                    #[doc = "**Active Camera**: No description provided"]
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
                    #[doc = "**Joints**: No description provided"]
                    pub fn joints() -> crate::Component< Vec<crate::EntityId> > {
                        *JOINTS
                    }
                }
            }
        }
    };

    let result = implementation(("kiwi.toml".to_string(), manifest.to_string()), true).unwrap();
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
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[allow(missing_docs)]
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the corresponding project."]
        pub mod components {}
    };

    let result = implementation(("kiwi.toml".to_string(), manifest.to_string()), false).unwrap();

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

        [components."a_cool_component_2"]
        type = "Empty"
        name = "Cool Component 2"
        description = "The cool-er component.\nBuy now in stores!"
        attributes = ["Store", "Networked"]
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[allow(missing_docs)]
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the corresponding project."]
        pub mod components {
            static A_COOL_COMPONENT: crate::LazyComponent<()> = crate::lazy_component!("my_project::a_cool_component");
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> crate::Component<()> {
                *A_COOL_COMPONENT
            }

            static A_COOL_COMPONENT_2: crate::LazyComponent<()> = crate::lazy_component!("my_project::a_cool_component_2");
            #[doc = "**Cool Component 2**: The cool-er component.\n\nBuy now in stores!\n\n*Attributes*: Store, Networked"]
            pub fn a_cool_component_2() -> crate::Component<()> {
                *A_COOL_COMPONENT_2
            }
        }
    };

    let result = implementation(("kiwi.toml".to_string(), manifest.to_string()), false).unwrap();

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
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[allow(missing_docs)]
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the corresponding project."]
        pub mod components {
            static A_COOL_COMPONENT: crate::LazyComponent<()> = crate::lazy_component!("evil_corp::my_project::a_cool_component");
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> crate::Component<()> {
                *A_COOL_COMPONENT
            }
        }
    };

    let result = implementation(("kiwi.toml".to_string(), manifest.to_string()), false).unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}
