use super::implementation;
use proc_macro2::Span;

fn api_name() -> syn::Path {
    let ident = syn::Ident::new("ambient_api2", Span::call_site());
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            #[doc = "**Core**"]
            pub mod core {
                use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                #[doc = "**App**"]
                pub mod app {
                    use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
                    use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
                    use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static JOINTS: Lazy< Component < Vec< ambient_api2::global::EntityId > > > = Lazy::new(|| __internal_get_component("core::rendering::joints"));
                    #[doc = "**Joints**: No description provided"]
                    pub fn joints() -> Component< Vec< ambient_api2::global::EntityId > > {
                        *JOINTS
                    }
                }
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {}
    };

    let result = implementation(
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            #[doc = "**Namespace**: A Test Namespace"]
            pub mod ns {
                use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
        (Some("kiwi.toml".to_string()), manifest.to_string()),
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
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
            pub mod core {
                use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                pub mod primitives {
                    use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
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
                    use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static COLOR: Lazy< Component<ambient_api2::global::Vec4> > = Lazy::new(|| __internal_get_component("my_project::core::rendering::color"));
                    #[doc = "**Color**"]
                    pub fn color() -> Component<ambient_api2::global::Vec4> { *COLOR }
                }
                pub mod transform {
                    use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    static ROTATION: Lazy< Component<ambient_api2::global::Quat> > = Lazy::new(|| __internal_get_component("my_project::core::transform::rotation"));
                    #[doc = "**Rotation**"]
                    pub fn rotation() -> Component<ambient_api2::global::Quat> { *ROTATION }
                    static SCALE: Lazy< Component<ambient_api2::global::Vec3> > = Lazy::new(|| __internal_get_component("my_project::core::transform::scale"));
                    #[doc = "**Scale**"]
                    pub fn scale() -> Component<ambient_api2::global::Vec3> { *SCALE }
                    static SPHERICAL_BILLBOARD: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::core::transform::spherical_billboard") );
                    #[doc = "**Spherical billboard**"]
                    pub fn spherical_billboard() -> Component<()> { *SPHERICAL_BILLBOARD }
                    static TRANSLATION: Lazy< Component<ambient_api2::global::Vec3> > = Lazy::new(|| __internal_get_component("my_project::core::transform::translation"));
                    #[doc = "**Translation**"]
                    pub fn translation() -> Component<ambient_api2::global::Vec3> { *TRANSLATION }
                }
            }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {
            use super::components;
            use ambient_api2::prelude::*;

            #[doc = "Makes a Colored Sphere (A sphere with some color!)"]
            pub fn make_colored_sphere() -> Components {
                Components::new()
                    .with_merge(make_sphere())
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
                    .with_merge(make_transformable())
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
        (Some("kiwi.toml".to_string()), manifest.to_string()),
        api_name(),
        false,
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_generate_concepts_with_all_supported_types() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        empty = { type = "Empty", name = "Empty", description = "" }
        bool = { type = "Bool", name = "Bool", description = "" }
        entity_id = { type = "EntityId", name = "EntityId", description = "" }
        f32 = { type = "F32", name = "F32", description = "" }
        f64 = { type = "F64", name = "F64", description = "" }
        mat4 = { type = "Mat4", name = "Mat4", description = "" }
        i32 = { type = "I32", name = "I32", description = "" }
        quat = { type = "Quat", name = "Quat", description = "" }
        string = { type = "String", name = "String", description = "" }
        u32 = { type = "U32", name = "U32", description = "" }
        u64 = { type = "U64", name = "U64", description = "" }
        vec2 = { type = "Vec2", name = "Vec2", description = "" }
        vec3 = { type = "Vec3", name = "Vec3", description = "" }
        vec4 = { type = "Vec4", name = "Vec4", description = "" }
        object_ref = { type = "ObjectRef", name = "ObjectRef", description = "" }
        vec_vec2 = { type = { type = "Vec", element_type = "Vec2" }, name = "VecVec2", description = "" }
        option_string1 = { type = { type = "Option", element_type = "String" }, name = "OptionString1", description = "" }
        option_string2 = { type = { type = "Option", element_type = "String" }, name = "OptionString2", description = "" }

        [concepts.everything]
        name = "Everything"
        description = "Everywhere all at once"
        [concepts.everything.components]
        empty = {}
        bool = true
        entity_id = "qmYJaglgRDwigkGXFFS9UQ"
        f32 = 3.14
        f64 = 3.14159
        mat4 = [1, 2, 3, 4, 5.0, 6.0, 7.0, 8.0, 9, 10, 11, 12, 13.0, 14.0, 15.0, 16.0]
        i32 = -4
        quat = [1.0, -0.5, 0.3, -0.6]
        string = "Everything"
        u32 = 100000
        u64 = "18446744073709551610"
        vec2 = [1.0, 2.0]
        vec3 = [1.0, 2.0, 3.0]
        vec4 = [1.0, 2.0, 3.0, 4.0]
        object_ref = "http://192.168.0.4:8999/assets/assets/Shape.glb/models/main.json"
        vec_vec2 = [[1.0, 2.0], [3.0, 4.0]]
        option_string1 = ["The Answer Is"]
        option_string2 = []
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};

            static BOOL: Lazy< Component<bool> > = Lazy::new(|| __internal_get_component("my_project::bool"));
            #[doc = "**Bool**"]
            pub fn bool() -> Component<bool> { *BOOL }

            static EMPTY: Lazy< Component<()> > = Lazy::new(|| __internal_get_component("my_project::empty"));
            #[doc = "**Empty**"]
            pub fn empty() -> Component<()> { *EMPTY }

            static ENTITY_ID: Lazy< Component<ambient_api2::global::EntityId> > = Lazy::new(|| __internal_get_component("my_project::entity_id"));
            #[doc = "**EntityId**"]
            pub fn entity_id() -> Component<ambient_api2::global::EntityId> { *ENTITY_ID }

            static F32: Lazy< Component<f32> > = Lazy::new(|| __internal_get_component("my_project::f32"));
            #[doc = "**F32**"]
            pub fn f32() -> Component<f32> { *F32 }

            static F64: Lazy< Component<f64> > = Lazy::new(|| __internal_get_component("my_project::f64"));
            #[doc = "**F64**"]
            pub fn f64() -> Component<f64> { *F64 }

            static I32: Lazy< Component<i32> > = Lazy::new(|| __internal_get_component("my_project::i32"));
            #[doc = "**I32**"]
            pub fn i32() -> Component<i32> { *I32 }

            static MAT4: Lazy< Component<ambient_api2::global::Mat4> > = Lazy::new(|| __internal_get_component("my_project::mat4"));
            #[doc = "**Mat4**"]
            pub fn mat4() -> Component<ambient_api2::global::Mat4> { *MAT4 }

            static OBJECT_REF: Lazy< Component<ambient_api2::global::ObjectRef> > = Lazy::new(|| __internal_get_component("my_project::object_ref"));
            #[doc = "**ObjectRef**"]
            pub fn object_ref() -> Component<ambient_api2::global::ObjectRef> { *OBJECT_REF }

            static OPTION_STRING1: Lazy< Component< Option<String> > > = Lazy::new(|| __internal_get_component("my_project::option_string1"));
            #[doc = "**OptionString1**"]
            pub fn option_string1() -> Component< Option<String> > { *OPTION_STRING1 }

            static OPTION_STRING2: Lazy< Component< Option<String> > > = Lazy::new(|| __internal_get_component("my_project::option_string2"));
            #[doc = "**OptionString2**"]
            pub fn option_string2() -> Component< Option<String> > { *OPTION_STRING2 }

            static QUAT: Lazy< Component<ambient_api2::global::Quat> > = Lazy::new(|| __internal_get_component("my_project::quat"));
            #[doc = "**Quat**"]
            pub fn quat() -> Component<ambient_api2::global::Quat> { *QUAT }

            static STRING: Lazy< Component<String> > = Lazy::new(|| __internal_get_component("my_project::string"));
            #[doc = "**String**"]
            pub fn string() -> Component<String> { *STRING }

            static U32: Lazy< Component<u32> > = Lazy::new(|| __internal_get_component("my_project::u32"));
            #[doc = "**U32**"]
            pub fn u32() -> Component<u32> { *U32 }

            static U64: Lazy< Component<u64> > = Lazy::new(|| __internal_get_component("my_project::u64"));
            #[doc = "**U64**"]
            pub fn u64() -> Component<u64> { *U64 }

            static VEC2: Lazy< Component<ambient_api2::global::Vec2> > = Lazy::new(|| __internal_get_component("my_project::vec2"));
            #[doc = "**Vec2**"]
            pub fn vec2() -> Component<ambient_api2::global::Vec2> { *VEC2 }

            static VEC3: Lazy< Component<ambient_api2::global::Vec3> > = Lazy::new(|| __internal_get_component("my_project::vec3"));
            #[doc = "**Vec3**"]
            pub fn vec3() -> Component<ambient_api2::global::Vec3> { *VEC3 }

            static VEC4: Lazy< Component<ambient_api2::global::Vec4> > = Lazy::new(|| __internal_get_component("my_project::vec4"));
            #[doc = "**Vec4**"]
            pub fn vec4() -> Component<ambient_api2::global::Vec4> { *VEC4 }

            static VEC_VEC2: Lazy< Component< Vec<ambient_api2::global::Vec2 > > > = Lazy::new(|| __internal_get_component("my_project::vec_vec2"));
            #[doc = "**VecVec2**"]
            pub fn vec_vec2() -> Component< Vec< ambient_api2::global::Vec2 > > { *VEC_VEC2 }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {
            use super::components;
            use ambient_api2::prelude::*;
            #[doc = "Makes a Everything (Everywhere all at once)"]
            pub fn make_everything() -> Components {
                Components::new()
                    .with(components::bool(), true)
                    .with(components::empty(), ())
                    .with(components::entity_id(), EntityId::from_base64("qmYJaglgRDwigkGXFFS9UQ"))
                    .with(components::f32(), 3.14f32)
                    .with(components::f64(), 3.14159f64)
                    .with(components::i32(), -4i32)
                    .with(components::mat4(), Mat4::from_cols_array(&[
                        1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32, 9f32, 10f32, 11f32, 12f32,
                        13f32, 14f32, 15f32, 16f32
                    ]))
                    .with(components::object_ref(), ObjectRef::new("http://192.168.0.4:8999/assets/assets/Shape.glb/models/main.json"))
                    .with(components::option_string1(), Some("The Answer Is".to_string()))
                    .with(components::option_string2(), None)
                    .with(components::quat(), Quat::from_xyzw(1f32, -0.5f32, 0.3f32, -0.6f32))
                    .with(components::string(), "Everything".to_string())
                    .with(components::u32(), 100000u32)
                    .with(components::u64(), 18446744073709551610u64)
                    .with(components::vec2(), Vec2::new(1f32, 2f32))
                    .with(components::vec3(), Vec3::new(1f32, 2f32, 3f32))
                    .with(components::vec4(), Vec4::new(1f32, 2f32, 3f32, 4f32))
                    .with(components::vec_vec2(), vec![Vec2::new(1f32, 2f32), Vec2::new(3f32, 4f32)])
            }
            #[doc = "Checks if the entity is a Everything (Everywhere all at once)"]
            pub fn is_everything(id: EntityId) -> bool {
                entity::has_components(
                    id,
                    &[
                        &components::bool(),
                        &components::empty(),
                        &components::entity_id(),
                        &components::f32(),
                        &components::f64(),
                        &components::i32(),
                        &components::mat4(),
                        &components::object_ref(),
                        &components::option_string1(),
                        &components::option_string2(),
                        &components::quat(),
                        &components::string(),
                        &components::u32(),
                        &components::u64(),
                        &components::vec2(),
                        &components::vec3(),
                        &components::vec4(),
                        &components::vec_vec2()
                    ]
                )
            }
        }
    };

    let result = implementation(
        (Some("kiwi.toml".to_string()), manifest.to_string()),
        api_name(),
        false,
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}

#[test]
fn can_extend_with_multiple_concepts() {
    let manifest = indoc::indoc! {r#"
        [project]
        id = "my_project"
        name = "My Project"

        [components]
        f32 = { type = "F32", name = "F32", description = "" }
        f64 = { type = "F64", name = "F64", description = "" }
        i32 = { type = "I32", name = "I32", description = "" }

        [concepts.concept1]
        name = "C1"
        description = ""
        components = { f32 = 4.0 }

        [concepts.concept2]
        name = "C2"
        description = ""
        components = { f64 = 8.0 }

        [concepts.concept3]
        name = "C3"
        description = ""
        extends = ["concept1", "concept2"]
        components = { i32 = 16 }
        "#};

    let expected_output = quote::quote! {
        const _PROJECT_MANIFEST: &'static str = include_str!("kiwi.toml");
        #[doc = r" Auto-generated component definitions. These come from `kiwi.toml` in the root of the project."]
        pub mod components {
            use ambient_api2::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};

            static F32: Lazy< Component<f32> > = Lazy::new(|| __internal_get_component("my_project::f32"));
            #[doc = "**F32**"]
            pub fn f32() -> Component<f32> { *F32 }

            static F64: Lazy< Component<f64> > = Lazy::new(|| __internal_get_component("my_project::f64"));
            #[doc = "**F64**"]
            pub fn f64() -> Component<f64> { *F64 }

            static I32: Lazy< Component<i32> > = Lazy::new(|| __internal_get_component("my_project::i32"));
            #[doc = "**I32**"]
            pub fn i32() -> Component<i32> { *I32 }
        }
        #[doc = r" Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept."]
        pub mod concepts {
            use super::components;
            use ambient_api2::prelude::*;

            #[doc = "Makes a C1 ()"]
            pub fn make_concept1() -> Components {
                Components::new().with(components::f32(), 4f32)
            }
            #[doc = "Checks if the entity is a C1 ()"]
            pub fn is_concept1(id: EntityId) -> bool {
                entity::has_components(id, &[&components::f32()])
            }

            #[doc = "Makes a C2 ()"]
            pub fn make_concept2() -> Components {
                Components::new().with(components::f64(), 8f64)
            }
            #[doc = "Checks if the entity is a C2 ()"]
            pub fn is_concept2(id: EntityId) -> bool {
                entity::has_components(id, &[&components::f64()])
            }

            #[doc = "Makes a C3 ()"]
            pub fn make_concept3() -> Components {
                Components::new()
                    .with_merge(make_concept1())
                    .with_merge(make_concept2())
                    .with(components::i32(), 16i32)
            }
            #[doc = "Checks if the entity is a C3 ()"]
            pub fn is_concept3(id: EntityId) -> bool {
                is_concept1(id) && is_concept2(id) && entity::has_components(id, &[&components::i32()])
            }
        }
    };

    let result = implementation(
        (Some("kiwi.toml".to_string()), manifest.to_string()),
        api_name(),
        false,
        false,
    )
    .unwrap();

    assert_eq!(result.to_string(), expected_output.to_string());
}
