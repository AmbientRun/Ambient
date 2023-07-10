extern crate proc_macro;

use ambient_project_semantic::{
    ArrayFileProvider, DiskFileProvider, ItemId, ItemMap, Scope, Semantic, Type, TypeInner,
};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub enum Context {
    Host,
    Guest {
        api_path: syn::Path,
        fully_qualified_path: bool,
    },
}

pub enum ManifestSource<'a> {
    Path(PathBuf),
    Array(&'a [(&'a str, &'a str)]),
}

pub fn generate_code(
    // bool is whether or not it's ambient
    manifests: Vec<(ManifestSource<'_>, bool)>,
    context: Context,
) -> anyhow::Result<TokenStream> {
    let mut semantic = Semantic::new()?;
    for (manifest, ambient) in manifests {
        match manifest {
            ManifestSource::Path(path) => {
                semantic.add_file(
                    &path,
                    &DiskFileProvider(path.parent().unwrap().to_owned()),
                    ambient,
                )?;
            }
            ManifestSource::Array(files) => {
                semantic.add_file(
                    Path::new("ambient.toml"),
                    &ArrayFileProvider { files },
                    ambient,
                )?;
            }
        }
    }

    // let mut printer = ambient_project_semantic::Printer::new();
    semantic.resolve()?;
    // printer.print(&semantic)?;

    let items = &semantic.items;
    let root_scope = &*semantic.root_scope();
    let type_map = {
        let mut type_map = HashMap::new();

        // First pass: add all root-level primitive types
        for type_id in root_scope.types.values() {
            let type_ = items.get(*type_id).expect("type id not in items");
            if let TypeInner::Primitive(pt) = type_.inner {
                let ty_tokens = syn::parse_str::<syn::Type>(&pt.to_string())?.to_token_stream();
                type_map.insert(*type_id, ty_tokens.clone());
                type_map.insert(items.get_vec_id(*type_id), quote! {Vec<#ty_tokens>});
                type_map.insert(items.get_option_id(*type_id), quote! {Option<#ty_tokens>});
            }
        }

        // Second pass: traverse the type graph and add all enums
        root_scope.visit_recursive(items, |scope| {
            for type_id in scope.types.values() {
                let type_ = items.get(*type_id).expect("type id not in items");
                if let TypeInner::Enum { .. } = type_.inner {
                    type_map.insert(*type_id, quote! {u32});
                }
            }
            Ok(())
        })?;

        type_map
    };

    let components = make_component_definitions(&context, &items, &type_map, root_scope)?;

    let output = quote! {
        /// Auto-generated component definitions. These come from `ambient.toml` in the root of the project.
        pub mod components {
            #components
        }
        /// Auto-generated concept definitions. Concepts are collections of components that describe some form of gameplay concept.
        ///
        /// They do not have any runtime representation outside of the components that compose them.
        pub mod concepts {}
        /// Auto-generated message definitions. Messages are used to communicate with the runtime, the other side of the network,
        /// and with other modules.
        pub mod messages {}
    };

    println!("{}", output.to_string());

    Ok(output)
}

fn make_component_definitions(
    context: &Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, proc_macro2::TokenStream>,
    root_scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let inner = make_component_definitions_inner(&context, &items, &type_map, root_scope)?;

    let namespaces = {
        let mut namespaces = vec![];
        root_scope.visit_recursive(items, |scope| {
            if !scope.components.is_empty() {
                namespaces.push(syn::parse_str::<syn::Path>(
                    &items.fully_qualified_display_path_rust_style(&*scope)?,
                )?);
            }
            Ok(())
        })?;
        namespaces
    };

    Ok(quote! {
        #inner

        pub fn init() {
            #(
                #namespaces::init_components();
            )*
        }
    })
}

fn make_component_definitions_inner(
    context: &Context,
    items: &ItemMap,
    type_map: &HashMap<ItemId<Type>, proc_macro2::TokenStream>,
    scope: &Scope,
) -> anyhow::Result<TokenStream> {
    let scopes = scope
        .scopes
        .values()
        .map(|s| {
            let scope = items.get(*s)?;
            let id = make_ident(&scope.data.id.as_ref().to_case(Case::Snake));

            let inner = make_component_definitions_inner(context, items, type_map, &scope)?;
            Ok(quote! {
                #[allow(unused)]
                pub mod #id {
                    #inner
                }
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let components = scope
        .components
        .values()
        .map(|c| {
            let component = items.get(*c)?;
            let id = component.data.id.as_ref().to_case(Case::Snake);
            let type_id = component.type_.as_resolved().expect("type was unresolved");
            let ty = type_map.get(&type_id).unwrap_or_else(|| {
                panic!(
                    "type not found: {}",
                    items
                        .fully_qualified_display_path_ambient_style(&*items.get(type_id).unwrap())
                        .unwrap()
                )
            });

            let attributes: Vec<_> = component
                .attributes
                .iter()
                .filter_map(|id| id.as_resolved())
                .map(|id| items.get(id).unwrap().data.id.as_ref().to_string())
                .collect();

            let name = component
                .name
                .as_ref()
                .map(|x| x as &str)
                .unwrap_or_else(|| component.data.id.as_ref());
            let mut doc_comment = format!("**{}**", name);

            if let Some(desc) = &component.description {
                if !desc.is_empty() {
                    doc_comment += &format!(": {}", desc.replace('\n', "\n\n"));
                }
            }

            // Metadata
            if !component.attributes.is_empty() {
                doc_comment += &format!("\n\n*Attributes*: {}", attributes.join(", "))
            }
            if let Some(default) = component.default.as_ref().and_then(|c| c.as_resolved()) {
                doc_comment += &format!("\n\n*Suggested Default*: {default}")
            }

            let doc_comment = doc_comment.trim();

            match context {
                Context::Host => {
                    let ident = make_ident(&id);
                    let attributes: Vec<_> = attributes
                        .into_iter()
                        .map(|s| make_ident(&s.to_case(Case::UpperCamel)))
                        .collect();
                    let description = component.description.to_owned().unwrap_or_default();

                    Ok(quote! {
                        #[doc = #doc_comment]
                        @[#(#attributes,)* Name[#name], Description[#description]]
                        #ident: #ty,
                    })
                }
                Context::Guest { .. } => {
                    let ident = make_ident(&id);
                    let uppercase_ident = make_ident(&id.to_uppercase());

                    let component_init = quote! {
                        Lazy::new(|| __internal_get_component(#ident))
                    };

                    Ok(quote! {
                        static #uppercase_ident: Lazy< Component< #ty > > = #component_init;

                        #[doc = #doc_comment]
                        fn #ident() -> Component<#ty> {
                            unimplemented!()
                        }
                    })
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let inner = if components.is_empty() {
        quote! {}
    } else {
        match context {
            Context::Host => {
                let namespace_path = items.fully_qualified_display_path_rust_style(scope)?;
                quote! {
                    use std::time::Duration;
                    use glam::{Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, Mat4, Quat};
                    use crate::{EntityId, Debuggable, Networked, Store, Resource, MaybeResource, Name, Description};
                    use ambient_shared_types::{ProceduralMeshHandle, ProceduralTextureHandle, ProceduralSamplerHandle, ProceduralMaterialHandle};
                    crate::components!(#namespace_path, {
                        #(#components)*
                    });
                }
            }
            Context::Guest {
                api_path,
                fully_qualified_path,
            } => {
                let fully_qualified_prefix = if *fully_qualified_path {
                    quote! { #api_path::global:: }
                } else {
                    quote! {}
                };
                quote! {
                    use #api_path::{once_cell::sync::Lazy, ecs::{Component, __internal_get_component}};
                    use #fully_qualified_prefix::{
                        EntityId, Mat4, Quat, Vec2, Vec3, Vec4, UVec2, UVec3, UVec4, IVec2, IVec3, IVec4,
                        Duration, ProceduralMeshHandle, ProceduralTextureHandle, ProceduralSamplerHandler,
                        ProceduralMaterialHandle
                    };
                    #(#components)*
                }
            }
        }
    };

    Ok(quote! {
        #(#scopes)*
        #inner
    })
}

fn make_ident(id: &str) -> syn::Ident {
    syn::Ident::new(id, proc_macro2::Span::call_site())
}
