use std::fmt::Debug;

use super::{
    tree::{TreeNode, TreeNodeInner},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;

/// Converts a tree to a token stream.
pub fn tree_to_token_stream<
    T: Clone + Debug,
    F: Fn(&Context, &TreeNode<T>, TokenStream) -> TokenStream + Copy,
>(
    // Current node of the tree (call with root)
    node: &TreeNode<T>,
    // The generation context
    context: &Context,
    // The wrapper to apply around the group of Others
    wrapper: F,
    // The function to call when converting a subtree. Should be a wrapper around this function.
    self_call: impl Fn(&TreeNode<T>, &Context, F) -> anyhow::Result<TokenStream>,
    // The function to call when converting the Other case (i.e. the actual value)
    other_call: impl Fn(&str, &T, &Context) -> anyhow::Result<TokenStream>,
) -> anyhow::Result<TokenStream> {
    let name = node.path.last().map(|s| s.as_ref()).unwrap_or_default();
    match &node.inner {
        TreeNodeInner::Namespace(ns) => {
            let (namespaces, others): (Vec<_>, Vec<_>) = ns
                .children
                .values()
                .partition(|child| matches!(child.inner, TreeNodeInner::Namespace(_)));

            let namespaces = namespaces
                .into_iter()
                .map(|child| self_call(child, context, wrapper))
                .collect::<Result<Vec<_>, _>>()?;

            let others = others
                .into_iter()
                .map(|child| self_call(child, context, wrapper))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(if name.is_empty() {
                let wrapped = if others.is_empty() {
                    TokenStream::new()
                } else {
                    wrapper(
                        context,
                        &node,
                        quote! {
                            #(#others)*
                        },
                    )
                };

                quote! {
                    #(#namespaces)*
                    #wrapped
                }
            } else {
                let name_ident: syn::Path = syn::parse_str(name)?;
                let doc_comment_fragment = ns.namespace.as_ref().map(|n| {
                    let mut doc_comment = format!("**{}**", n.name);
                    if !n.description.is_empty() {
                        doc_comment += &format!(": {}", n.description.replace('\n', "\n\n"));
                    }

                    quote! {
                        #[doc = #doc_comment]
                    }
                });

                let wrapped = if others.is_empty() {
                    TokenStream::new()
                } else {
                    wrapper(
                        context,
                        &node,
                        quote! {
                            #(#others)*
                        },
                    )
                };
                quote! {
                    #doc_comment_fragment
                    pub mod #name_ident {
                        #(#namespaces)*
                        #wrapped
                    }
                }
            })
        }
        TreeNodeInner::Other(other) => other_call(name, other, context),
    }
}
