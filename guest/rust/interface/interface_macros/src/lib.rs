extern crate proc_macro;

use std::path::PathBuf;

use anyhow::Context;
use proc_macro::TokenStream;
use quote::quote;

/// Makes your main() function accessible to the scripting host.
///
/// If you do not add this attribute to your main() function, your script will not run.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_name = item.sig.ident.clone();
    if item.sig.asyncness.is_none() {
        panic!("the `{fn_name}` function must be async");
    }

    quote! {
        #item

        #[no_mangle]
        pub extern "C" fn call_main(runtime_interface_version: u32) {
            if INTERFACE_VERSION != runtime_interface_version {
                panic!("This script was compiled with interface version {{INTERFACE_VERSION}}, but the script host is running with version {{runtime_interface_version}}");
            }
            run_async(#fn_name());
        }
    }.into()
}

/// Parses your project's manifest and generates components and other boilerplate.
#[proc_macro]
pub fn elements_project(input: TokenStream) -> TokenStream {
    let extend_paths: Option<Vec<Vec<String>>> = if input.is_empty() {
        None
    } else {
        syn::custom_keyword!(extend);

        struct Extend {
            elems: syn::punctuated::Punctuated<syn::Path, syn::token::Comma>,
        }
        impl syn::parse::Parse for Extend {
            fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
                let _extend_token = input.parse::<extend>()?;
                let _equal_token = input.parse::<syn::Token![=]>()?;

                let content;
                let _bracket_token = syn::bracketed!(content in input);
                let mut elems = syn::punctuated::Punctuated::new();

                while !content.is_empty() {
                    let first: syn::Path = content.parse()?;
                    elems.push_value(first);
                    if content.is_empty() {
                        break;
                    }
                    let punct = content.parse()?;
                    elems.push_punct(punct);
                }

                Ok(Self { elems })
            }
        }

        let extend = syn::parse_macro_input!(input as Extend);
        Some(
            extend
                .elems
                .into_iter()
                .map(|p| {
                    p.segments
                        .into_iter()
                        .map(|s| s.ident.to_string())
                        .collect()
                })
                .collect(),
        )
    };

    TokenStream::from(
        elements_project_impl(
            elements_project_read_file("elements.toml".to_string()).unwrap(),
            extend_paths
                .as_ref()
                .map(|a| a.as_slice())
                .unwrap_or_default(),
            extend_paths.is_some(),
        )
        .unwrap(),
    )
}

fn elements_project_read_file(file_path: String) -> anyhow::Result<(String, String)> {
    let file_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("no manifest dir")?)
        .join(&file_path);
    let file_path_str = format!("{}", file_path.display());

    let contents = std::fs::read_to_string(&file_path)?;

    Ok((file_path_str, contents))
}

fn elements_project_impl(
    (file_path, contents): (String, String),
    extend_paths: &[Vec<String>],
    global_namespace: bool,
) -> anyhow::Result<proc_macro2::TokenStream> {
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Deserialize, Debug)]
    struct Manifest {
        project: Project,
        components: BTreeMap<String, Component>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Project {
        id: String,
        organization: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    struct Component {
        name: String,
        description: String,
        #[serde(rename = "type")]
        type_: ComponentType,
    }

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum ComponentType {
        String(String),
        ContainerType {
            #[serde(rename = "type")]
            type_: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            element_type: Option<String>,
        },
    }
    impl ComponentType {
        fn convert_primitive_type_to_rust_type(ty: &str) -> Option<proc_macro2::TokenStream> {
            match ty {
                "Empty" => Some(quote! {()}),
                "Bool" => Some(quote! {bool}),
                "EntityId" => Some(quote! {crate::EntityId}),
                "F32" => Some(quote! {f32}),
                "F64" => Some(quote! {f64}),
                "Mat4" => Some(quote! {crate::Mat4}),
                "I32" => Some(quote! {i32}),
                "Quat" => Some(quote! {crate::Quat}),
                "String" => Some(quote! {String}),
                "U32" => Some(quote! {u32}),
                "U64" => Some(quote! {u64}),
                "Vec2" => Some(quote! {crate::Vec2}),
                "Vec3" => Some(quote! {crate::Vec3}),
                "Vec4" => Some(quote! {crate::Vec4}),
                "ObjectRef" => Some(quote! {crate::ObjectRef}),
                "EntityUid" => Some(quote! {crate::EntityUid}),
                _ => None,
            }
        }

        fn convert_container_type_to_rust_type(ty: &str) -> Option<proc_macro2::TokenStream> {
            match ty {
                "Vec" => Some(quote! {Vec}),
                "Option" => Some(quote! {Option}),
                _ => None,
            }
        }

        fn to_token_stream(&self) -> anyhow::Result<proc_macro2::TokenStream> {
            match self {
                ComponentType::String(ty) => {
                    Self::convert_primitive_type_to_rust_type(ty).context("invalid primitive type")
                }
                ComponentType::ContainerType {
                    type_,
                    element_type,
                } => {
                    let container_ty = Self::convert_container_type_to_rust_type(&type_)
                        .context("invalid container type")?;
                    let element_ty = element_type
                        .as_deref()
                        .map(Self::convert_primitive_type_to_rust_type)
                        .context("invalid element type")?;
                    Ok(if let Some(element_ty) = element_ty {
                        quote! { #container_ty < #element_ty > }
                    } else {
                        container_ty
                    })
                }
            }
        }
    }

    let manifest: Manifest = toml::from_str(&contents)?;

    #[derive(Debug)]
    enum TreeNodeInner {
        Module(BTreeMap<String, TreeNode>),
        Component(Component),
        UseAll(Vec<String>),
    }

    #[derive(Debug)]
    struct TreeNode {
        path: Vec<String>,
        inner: TreeNodeInner,
    }
    impl TreeNode {
        fn new(path: Vec<String>, inner: TreeNodeInner) -> Self {
            Self { path, inner }
        }
    }

    let mut root = BTreeMap::new();
    fn insert_into_root(
        root: &mut BTreeMap<String, TreeNode>,
        segments: &[String],
        inner: TreeNodeInner,
    ) -> anyhow::Result<()> {
        let mut manifest_head = root;
        let (leaf_id, modules) = segments.split_last().context("empty segments")?;

        let mut segments_so_far = vec![];
        for segment in modules {
            segments_so_far.push(segment.to_string());

            let new_head = manifest_head
                .entry(segment.to_string())
                .or_insert(TreeNode::new(
                    segments_so_far.clone(),
                    TreeNodeInner::Module(Default::default()),
                ));

            manifest_head = match &mut new_head.inner {
                TreeNodeInner::Module(module) => module,
                _ => anyhow::bail!("found a non-module where a module was expected"),
            };
        }

        manifest_head.insert(
            leaf_id.clone(),
            TreeNode::new(segments.into_iter().map(|s| s.to_string()).collect(), inner),
        );

        Ok(())
    }
    for (id, component) in manifest.components {
        insert_into_root(
            &mut root,
            &id.split("::").map(|s| s.to_string()).collect::<Vec<_>>(),
            TreeNodeInner::Component(component),
        )?;
    }
    for path in extend_paths {
        let components_index = path
            .iter()
            .position(|s| s == "components")
            .context("expected components:: in extend path")?;
        let mut subpath = path[components_index + 1..path.len()].to_vec();
        subpath.push("#use_all#".to_string());

        insert_into_root(&mut root, &subpath, TreeNodeInner::UseAll(path.clone()))?;
    }

    fn expand_tree(
        tree_node: &TreeNode,
        project_path: &[String],
    ) -> anyhow::Result<proc_macro2::TokenStream> {
        let name = tree_node
            .path
            .last()
            .map(|s| s.as_str())
            .unwrap_or_default();
        match &tree_node.inner {
            TreeNodeInner::Module(module) => {
                let children = module
                    .values()
                    .map(|child| expand_tree(child, project_path))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(if name.is_empty() {
                    quote! {
                        #(#children)*
                    }
                } else {
                    let name_ident: syn::Ident = syn::parse_str(name)?;
                    quote! {
                        pub mod #name_ident {
                            #(#children)*
                        }
                    }
                })
            }
            TreeNodeInner::Component(component) => {
                let name_ident: syn::Ident = syn::parse_str(name)?;
                let name_uppercase_ident: syn::Ident = syn::parse_str(&name.to_ascii_uppercase())?;
                let component_ty = component.type_.to_token_stream()?;
                let doc_comment = format!("{}: {}", component.name, component.description);
                let id = [project_path, &tree_node.path].concat().join("::");

                Ok(quote! {
                    static #name_uppercase_ident: crate::LazyComponent<#component_ty> = crate::lazy_component!(#id);
                    #[doc = #doc_comment]
                    pub fn #name_ident() -> crate::Component<#component_ty> { *#name_uppercase_ident }
                })
            }
            TreeNodeInner::UseAll(path) => {
                let path = path
                    .into_iter()
                    .map(|s| syn::parse_str::<syn::Ident>(&s))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(quote! {
                    pub use #(#path::)* *;
                })
            }
        }
    }

    let project_path: Vec<_> = if global_namespace {
        vec![]
    } else {
        manifest
            .project
            .organization
            .iter()
            .chain(std::iter::once(&manifest.project.id))
            .cloned()
            .collect()
    };
    let expanded_tree = expand_tree(
        &TreeNode::new(vec![], TreeNodeInner::Module(root)),
        &project_path,
    )?;
    Ok(quote!(
        const _PROJECT_MANIFEST: &'static str = include_str!(#file_path);
        #[allow(missing_docs)]
        pub mod components {
            #expanded_tree
        }
    ))
}

#[cfg(test)]
mod tests {
    use crate::elements_project_impl;

    #[test]
    fn can_generate_components_from_manifest_in_global_namespace() {
        let manifest = indoc::indoc! {r#"
        [project]
        name = "runtime_components"

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
                        #[doc = "Main Scene: "]
                        pub fn main_scene() -> crate::Component<()> {
                            *MAIN_SCENE
                        }
                        static NAME: crate::LazyComponent<String> = crate::lazy_component!("core::app::name");
                        #[doc = "name: "]
                        pub fn name() -> crate::Component<String> {
                            *NAME
                        }
                    }
                    pub mod camera {
                        static ACTIVE_CAMERA: crate::LazyComponent<f32> = crate::lazy_component!("core::camera::active_camera");
                        #[doc = "Active Camera: No description provided"]
                        pub fn active_camera() -> crate::Component<f32> {
                            *ACTIVE_CAMERA
                        }
                        static ASPECT_RATIO: crate::LazyComponent<f32> = crate::lazy_component!("core::camera::aspect_ratio");
                        #[doc = "Aspect Ratio: "]
                        pub fn aspect_ratio() -> crate::Component<f32> {
                            *ASPECT_RATIO
                        }
                    }
                    pub mod rendering {
                        static JOINTS: crate::LazyComponent< Vec<crate::EntityId> > = crate::lazy_component!("core::rendering::joints");
                        #[doc = "Joints: No description provided"]
                        pub fn joints() -> crate::Component< Vec<crate::EntityId> > {
                            *JOINTS
                        }
                    }
                }
            }
        };

        let result = elements_project_impl(
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
        name = "runtime_components"

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
                        #[doc = "Main Scene: "]
                        pub fn main_scene() -> crate::Component<()> {
                            *MAIN_SCENE
                        }
                    }
                    pub mod camera {
                        pub use base::components::core::camera::*;
                        static ACTIVE_CAMERA: crate::LazyComponent<f32> = crate::lazy_component!("core::camera::active_camera");
                        #[doc = "Active Camera: No description provided"]
                        pub fn active_camera() -> crate::Component<f32> {
                            *ACTIVE_CAMERA
                        }
                    }
                    pub mod player {
                        pub use base::components::core::player::*;
                    }
                    pub mod rendering {
                        static JOINTS: crate::LazyComponent< Vec<crate::EntityId> > = crate::lazy_component!("core::rendering::joints");
                        #[doc = "Joints: No description provided"]
                        pub fn joints() -> crate::Component< Vec<crate::EntityId> > {
                            *JOINTS
                        }
                    }
                }
            }
        };

        let result = elements_project_impl(
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
    fn can_generate_components_from_manifest() {
        let manifest = indoc::indoc! {r#"
        [project]
        name = "my_project"

        [components]
        a_cool_component = { name = "Cool Component", description = "", type = "Empty" }
        "#};

        let expected_output = quote::quote! {
            const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
            #[allow(missing_docs)]
            pub mod components {
                static A_COOL_COMPONENT: crate::LazyComponent<()> = crate::lazy_component!("my_project::a_cool_component");
                #[doc = "Cool Component: "]
                pub fn a_cool_component() -> crate::Component<()> {
                    *A_COOL_COMPONENT
                }
            }
        };

        let result = elements_project_impl(
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
        name = "my_project"
        organization = "evil_corp"

        [components]
        a_cool_component = { name = "Cool Component", description = "", type = "Empty" }
        "#};

        let expected_output = quote::quote! {
            const _PROJECT_MANIFEST: &'static str = include_str!("elementsy.toml");
            #[allow(missing_docs)]
            pub mod components {
                static A_COOL_COMPONENT: crate::LazyComponent<()> = crate::lazy_component!("evil_corp::my_project::a_cool_component");
                #[doc = "Cool Component: "]
                pub fn a_cool_component() -> crate::Component<()> {
                    *A_COOL_COMPONENT
                }
            }
        };

        let result = elements_project_impl(
            ("elementsy.toml".to_string(), manifest.to_string()),
            &[],
            false,
        )
        .unwrap();

        assert_eq!(result.to_string(), expected_output.to_string());
    }
}
