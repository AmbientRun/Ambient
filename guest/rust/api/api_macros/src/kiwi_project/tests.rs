use super::implementation;
use proc_macro2::Span;

fn api_name() -> syn::Path {
    let ident = syn::Ident::new("kiwi_api2", Span::call_site());
    ident.into()
}

#[test]
fn can_generate_components_from_manifest_in_global_namespace() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "runtime_components"
        name = "Runtime Components"

        [components]
        "core" = { name = "Core", description = "" }
        "core::app" = { name = "App", description = "" }
        "core::camera" = { name = "Camera", description = "" }
        "core::rendering" = { name = "Rendering", description = "" }

        "core::app::main_scene" = { name = "Main Scene", description = "", type = "Empty" }
        "core::app::name" = { name = "name", description = "", type = "String" }
        "core::camera::active_camera" = { name = "Active Camera", description = "No description provided", type = "F32" }
        "core::camera::aspect_ratio" = { name = "Aspect Ratio", description = "", type = "F32" }
        "core::rendering::joints" = { name = "Joints", description = "No description provided", type = { type = "Vec", element_type = "EntityId" } }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            #[doc = "**Core**"]
            pub mod core {
                use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                #[doc = "**App**"]
                pub mod app {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static MAIN_SCENE: Lazy<Component<()>> = Lazy::new(|| __internal_get_component("core::app::main_scene"));
                    #[doc = "**Main Scene**"]
                    pub fn main_scene() -> Component<()> {
                        *MAIN_SCENE
                    }
                    static NAME: Lazy<Component<String>> = Lazy::new(|| __internal_get_component("core::app::name"));
                    #[doc = "**name**"]
                    pub fn name() -> Component<String> {
                        *NAME
                    }
                }
                #[doc = "**Camera**"]
                pub mod camera {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static ACTIVE_CAMERA: Lazy<Component<f32>> = Lazy::new(|| __internal_get_component("core::camera::active_camera"));
                    #[doc = "**Active Camera**: No description provided"]
                    pub fn active_camera() -> Component<f32> {
                        *ACTIVE_CAMERA
                    }
                    static ASPECT_RATIO: Lazy<Component<f32>> = Lazy::new(|| __internal_get_component("core::camera::aspect_ratio"));
                    #[doc = "**Aspect Ratio**"]
                    pub fn aspect_ratio() -> Component<f32> {
                        *ASPECT_RATIO
                    }
                }
                #[doc = "**Rendering**"]
                pub mod rendering {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static JOINTS: Lazy<Component<Vec<kiwi_api2::global::EntityId> >> = Lazy::new(|| __internal_get_component("core::rendering::joints"));
                    #[doc = "**Joints**: No description provided"]
                    pub fn joints() -> Component< Vec<kiwi_api2::global::EntityId> > {
                        *JOINTS
                    }
                }
            }
        }
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
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
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
        }
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
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

        [components."a_cool_component_2"]
        type = "Empty"
        name = "Cool Component 2"
        description = "The cool-er component.\nBuy now in stores!"
        attributes = ["Store", "Networked"]
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            static A_COOL_COMPONENT: Lazy<Component<()>> = Lazy::new(|| __internal_get_component("my_project::a_cool_component"));
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> Component<()> {
                *A_COOL_COMPONENT
            }

            static A_COOL_COMPONENT_2: Lazy<Component<()>> = Lazy::new(|| __internal_get_component("my_project::a_cool_component_2"));
            #[doc = "**Cool Component 2**: The cool-er component.\n\nBuy now in stores!\n\n*Attributes*: Store, Networked"]
            pub fn a_cool_component_2() -> Component<()> {
                *A_COOL_COMPONENT_2
            }
        }
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
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
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            static A_COOL_COMPONENT: Lazy<Component<()>> = Lazy::new(|| __internal_get_component("evil_corp::my_project::a_cool_component"));
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> Component<()> {
                *A_COOL_COMPONENT
            }
        }
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_generate_components_with_documented_namespace_from_manifest() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        "ns::a_cool_component" = { name = "Cool Component", description = "Cool!", type = "Empty" }
        "ns" = { name = "Namespace", description = "A Test Namespace" }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            #[doc = "**Namespace**: A Test Namespace"]
            pub mod ns {
                use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                static A_COOL_COMPONENT: Lazy<Component<()>> = Lazy::new(|| __internal_get_component("my_project::ns::a_cool_component"));
                #[doc = "**Cool Component**: Cool!"]
                pub fn a_cool_component() -> Component<()> {
                    *A_COOL_COMPONENT
                }
            }
        }
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn will_error_on_undocumented_namespace() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        "ns::a_cool_component" = { name = "Cool Component", description = "Cool!", type = "Empty" }
        "#};

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
    );

    assert_eq!(
        result.unwrap_err().to_string(),
        "The namespace `ns` is missing a name and description."
    );
}
