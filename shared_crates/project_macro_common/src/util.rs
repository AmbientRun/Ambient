use std::fmt::Debug;

use super::{
    tree::{TreeNode, TreeNodeInner},
    Context,
};
use proc_macro2::TokenStream;
use quote::quote;

/// Converts a tree to a token stream.
pub fn tree_to_token_stream<T: Clone + Debug>(
    // Current node of the tree (call with root)
    node: &TreeNode<T>,
    // The generation context
    context: &Context,
    // The prelude to insert for namespaces
    prelude: &TokenStream,
    // The function to call when converting a subtree. Should be a wrapper around this function.
    self_call: impl Fn(&TreeNode<T>, &Context, &TokenStream) -> anyhow::Result<TokenStream>,
    // The function to call when converting the Other case (i.e. the actual value)
    other_call: impl Fn(&str, &T, &Context) -> anyhow::Result<TokenStream>,
) -> anyhow::Result<TokenStream> {
    let name = node.path.last().map(|s| s.as_ref()).unwrap_or_default();
    match &node.inner {
        TreeNodeInner::Namespace(ns) => {
            let children = ns
                .children
                .values()
                .map(|child| self_call(child, context, prelude))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(if name.is_empty() {
                quote! {
                    #prelude
                    #(#children)*
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
                quote! {
                    #doc_comment_fragment
                    pub mod #name_ident {
                        #prelude
                        #(#children)*
                    }
                }
            })
        }
        TreeNodeInner::Other(other) => other_call(name, other, context),
    }
}
