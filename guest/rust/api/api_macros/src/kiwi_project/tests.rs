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
                    static MAIN_SCENE: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("core::app::main_scene"));
                    #[doc = "**Main Scene**"]
                    pub fn main_scene() -> Component<()> {
                        *MAIN_SCENE
                    }
                    static NAME: Lazy< Component<String> > = Lazy::new(|| __internal_get_component("core::app::name"));
                    #[doc = "**name**"]
                    pub fn name() -> Component<String> {
                        *NAME
                    }
                }
                #[doc = "**Camera**"]
                pub mod camera {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static ACTIVE_CAMERA: Lazy< Component<f32> > = Lazy::new(|| __internal_get_component("core::camera::active_camera"));
                    #[doc = "**Active Camera**: No description provided"]
                    pub fn active_camera() -> Component<f32> {
                        *ACTIVE_CAMERA
                    }
                    static ASPECT_RATIO: Lazy< Component<f32> > = Lazy::new(|| __internal_get_component("core::camera::aspect_ratio"));
                    #[doc = "**Aspect Ratio**"]
                    pub fn aspect_ratio() -> Component<f32> {
                        *ASPECT_RATIO
                    }
                }
                #[doc = "**Rendering**"]
                pub mod rendering {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static JOINTS: Lazy< Component < Vec< kiwi_api2::global::EntityId > > > = Lazy::new(|| __internal_get_component("core::rendering::joints"));
                    #[doc = "**Joints**: No description provided"]
                    pub fn joints() -> Component< Vec< kiwi_api2::global::EntityId > > {
                        *JOINTS
                    }
                }
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        true,
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
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
        true,
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
        type = "String"
        name = "Cool Component 2"
        description = "The cool-er component.\nBuy now in stores!"
        attributes = ["Store", "Networked"]
        default = "The Coolest"
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            static A_COOL_COMPONENT: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::a_cool_component"));
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> Component<()> {
                *A_COOL_COMPONENT
            }

            static A_COOL_COMPONENT_2: Lazy< Component<String> > = Lazy::new(|| __internal_get_component("my_project::a_cool_component_2"));
            #[doc = "**Cool Component 2**: The cool-er component.\n\nBuy now in stores!\n\n*Attributes*: Store, Networked\n\n*Suggested Default*: \"The Coolest\""]
            pub fn a_cool_component_2() -> Component<String> {
                *A_COOL_COMPONENT_2
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
        true,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_generate_component_with_contained_type() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        a_cool_component = { name = "Cool Component", description = "", type = { type = "Empty" } }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            static A_COOL_COMPONENT: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::a_cool_component"));
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> Component<()> {
                *A_COOL_COMPONENT
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
        true,
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
            static A_COOL_COMPONENT: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("evil_corp::my_project::a_cool_component"));
            #[doc = "**Cool Component**"]
            pub fn a_cool_component() -> Component<()> {
                *A_COOL_COMPONENT
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
        true,
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
                static A_COOL_COMPONENT: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::ns::a_cool_component"));
                #[doc = "**Cool Component**: Cool!"]
                pub fn a_cool_component() -> Component<()> {
                    *A_COOL_COMPONENT
                }
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
        true,
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
        true,
    );

    assert_eq!(
        result.unwrap_err().to_string(),
        "The namespace `ns` is missing a name and description."
    );
}

#[test]
fn can_generate_concepts() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        "core::transform::rotation" = { type = "Quat", name = "Rotation", description = "" }
        "core::transform::scale" = { type = "Vec3", name = "Scale", description = "" }
        "core::transform::spherical_billboard" = { type = "Empty", name = "Spherical billboard", description = "" }
        "core::transform::translation" = { type = "Vec3", name = "Translation", description = "" }
        "core::primitives::sphere" = { type = "Empty", name = "Sphere", description = "" }
        "core::primitives::sphere_radius" = { type = "F32", name = "Sphere radius", description = "" }
        "core::primitives::sphere_sectors" = { type = "U32", name = "Sphere sectors", description = "" }
        "core::primitives::sphere_stacks" = { type = "U32", name = "Sphere stacks", description = "" }
        "core::rendering::color" = { type = "Vec4", name = "Color", description = "" }

        [concepts.transformable]
        name = "Transformable"
        description = "Can be translated, rotated and scaled."
        [concepts.transformable.components]
        "core::transform::translation" = [0, 0, 0]
        "core::transform::rotation" = [0, 0, 0, 1]
        "core::transform::scale" = [1, 1, 1]

        [concepts.sphere]
        name = "Sphere"
        description = "A primitive sphere."
        extends = ["transformable"]
        [concepts.sphere.components]
        "core::primitives::sphere" = {}
        "core::primitives::sphere_radius" = 0.5
        "core::primitives::sphere_sectors" = 36
        "core::primitives::sphere_stacks" = 18

        [concepts.colored_sphere]
        name = "Colored Sphere"
        description = "A sphere with some color!"
        extends = ["sphere"]
        components = { "core::rendering::color" = [1, 1, 1, 1] }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            pub mod core {
                use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                pub mod primitives {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static SPHERE: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::core::primitives::sphere"));
                    #[doc = "**Sphere**"]
                    pub fn sphere() -> Component<()> { *SPHERE }
                    static SPHERE_RADIUS: Lazy< Component<f32> > = Lazy::new(|| __internal_get_component("my_project::core::primitives::sphere_radius"));
                    #[doc = "**Sphere radius**"]
                    pub fn sphere_radius() -> Component<f32> { *SPHERE_RADIUS }
                    static SPHERE_SECTORS: Lazy< Component<u32> > = Lazy::new(|| __internal_get_component("my_project::core::primitives::sphere_sectors"));
                    #[doc = "**Sphere sectors**"]
                    pub fn sphere_sectors() -> Component<u32> { *SPHERE_SECTORS }
                    static SPHERE_STACKS: Lazy< Component<u32> > = Lazy::new(|| __internal_get_component("my_project::core::primitives::sphere_stacks"));
                    #[doc = "**Sphere stacks**"]
                    pub fn sphere_stacks() -> Component<u32> { *SPHERE_STACKS }
                }
                pub mod rendering {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static COLOR: Lazy< Component<kiwi_api2::global::Vec4> > = Lazy::new(|| __internal_get_component("my_project::core::rendering::color"));
                    #[doc = "**Color**"]
                    pub fn color() -> Component<kiwi_api2::global::Vec4> { *COLOR }
                }
                pub mod transform {
                    use kiwi_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static ROTATION: Lazy< Component<kiwi_api2::global::Quat> > = Lazy::new(|| __internal_get_component("my_project::core::transform::rotation"));
                    #[doc = "**Rotation**"]
                    pub fn rotation() -> Component<kiwi_api2::global::Quat> { *ROTATION }
                    static SCALE: Lazy< Component<kiwi_api2::global::Vec3> > = Lazy::new(|| __internal_get_component("my_project::core::transform::scale"));
                    #[doc = "**Scale**"]
                    pub fn scale() -> Component<kiwi_api2::global::Vec3> { *SCALE }
                    static SPHERICAL_BILLBOARD: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::core::transform::spherical_billboard") );
                    #[doc = "**Spherical billboard**"]
                    pub fn spherical_billboard() -> Component<()> { *SPHERICAL_BILLBOARD }
                    static TRANSLATION: Lazy< Component<kiwi_api2::global::Vec3> > = Lazy::new(|| __internal_get_component("my_project::core::transform::translation"));
                    #[doc = "**Translation**"]
                    pub fn translation() -> Component<kiwi_api2::global::Vec3> { *TRANSLATION }
                }
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {
            use super::components;
            use kiwi_api2::prelude::*;

            #[doc = "Makes a Colored Sphere (A sphere with some color!)"]
            pub fn make_colored_sphere() -> Components {
                Components::new()
                    .merge(make_sphere())
                    .with(components::core::rendering::color(), Vec4::new(1f32, 1f32, 1f32, 1f32))
            }

            #[doc = "Checks if the entity is a Colored Sphere (A sphere with some color!)"]
            pub fn is_colored_sphere(id: EntityId) -> bool {
                is_sphere(id) && entity::has_components(id, &[
                    &components::core::rendering::color()
                ])
            }

            #[doc = "Makes a Sphere (A primitive sphere.)"]
            pub fn make_sphere() -> Components {
                Components::new()
                    .merge(make_transformable())
                    .with(components::core::primitives::sphere(), ())
                    .with(components::core::primitives::sphere_radius(), 0.5f32)
                    .with(components::core::primitives::sphere_sectors(), 36u32)
                    .with(components::core::primitives::sphere_stacks(), 18u32)
            }

            #[doc = "Checks if the entity is a Sphere (A primitive sphere.)"]
            pub fn is_sphere(id: EntityId) -> bool {
                is_transformable(id) && entity::has_components(id, &[
                    &components::core::primitives::sphere(),
                    &components::core::primitives::sphere_radius(),
                    &components::core::primitives::sphere_sectors(),
                    &components::core::primitives::sphere_stacks()
                ])
            }

            #[doc = "Makes a Transformable (Can be translated, rotated and scaled.)"]
            pub fn make_transformable() -> Components {
                Components::new()
                    .with(components::core::transform::rotation(), Quat::from_xyzw(0f32, 0f32, 0f32, 1f32))
                    .with(components::core::transform::scale(), Vec3::new(1f32, 1f32, 1f32))
                    .with(components::core::transform::translation(), Vec3::new(0f32, 0f32, 0f32))
            }

            #[doc = "Checks if the entity is a Transformable (Can be translated, rotated and scaled.)"]
            pub fn is_transformable(id: EntityId) -> bool {
                entity::has_components(id, &[
                    &components::core::transform::rotation(),
                    &components::core::transform::scale(),
                    &components::core::transform::translation()
                ])
            }
        }
    };

    let result = implementation(
        ("kiwi.toml".to_string(), manifest.to_string()),
        api_name(),
        false,
        false,
    )
    .unwrap();

    println!("\n\n{result}\n\n");

    assert_eq!(result.to_string(), expected_output.to_string());
}

// TODO: write test for non-global case, extending multiple concepts
// TODO: write test for different types, including vec/opt
