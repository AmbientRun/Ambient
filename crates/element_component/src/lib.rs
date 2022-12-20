//! Provides a macro for converting a function to an `ElementComponent`, allowing for more concise definitions of components.

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// Helper macro to implement a `ElementComponent` with a pure free function.
///
/// Rewrites a `fn Component(&mut World, &mut Hooks, ...Args) -> Element` to a `struct Component` and an implementation
/// of `ElementComponent` for that `Component`, where the trait's `render` function corresponds to this function.
///
/// Example:
///
/// ```ignore
/// pub fn FancyText(
///     world: &mut elements_ecs::World,
///     hooks: &mut elements_element::Hooks,
///     /// The message to display
///     msg: String,
///     alpha: f32,
/// ) -> elements_element::Element {
///     Text::el(msg)
/// }
/// ```
///
/// becomes:
///
/// ```ignore
/// #[derive(std::clone::Clone, std::fmt::Debug)]
/// pub struct FancyText {
///     pub
///     /// The message to display
///     msg: String,
///     pub alpha: f32,
/// }
/// impl elements_element::ElementComponent for FancyText {
///     fn render(self: Box<Self>, world: &mut elements_ecs::World, hooks: &mut elements_element::Hooks) -> elements_element::Element {
///         let Self { msg, alpha } = *self;
///         {
///             Text::el(msg)
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn element_component(input: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_derive_element_component(input.into(), item.into()).into()
}

fn do_derive_element_component(input: TokenStream, item: TokenStream) -> TokenStream {
    let with_el = if input.is_empty() {
        true
    } else {
        let input = match syn::parse2::<syn::Ident>(input) {
            Ok(ident) => ident,
            Err(err) => return err.to_compile_error(),
        };

        if input == "without_el" {
            false
        } else {
            panic!("unexpected argument {input} to macro");
        }
    };

    let item = match syn::parse2::<syn::ItemFn>(item.clone()) {
        Ok(syntax_tree) => syntax_tree,
        Err(err) => return err.to_compile_error(),
    };

    fn get_pat_type(arg: &syn::FnArg) -> Option<&syn::PatType> {
        match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(t) => Some(t),
        }
    }

    if item.sig.receiver().is_some() {
        panic!("a `self` was specified in `{}`; your function must be a free function for this macro to work", item.sig.to_token_stream());
    }

    let attrs = item.attrs;
    let name = item.sig.ident;
    let ret = item.sig.output;

    let (generic_params, generic_idents) = {
        let generic_params = item.sig.generics.params.into_iter().collect_vec();
        let generic_idents = generic_params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Type(t) => t.ident.to_token_stream(),
                syn::GenericParam::Lifetime(lt) => lt.lifetime.to_token_stream(),
                syn::GenericParam::Const(c) => c.ident.to_token_stream(),
            })
            .collect_vec();

        let generic_params = if generic_params.is_empty() {
            quote! {}
        } else {
            quote! {<#(#generic_params),*>}
        };

        let generic_idents = if generic_idents.is_empty() {
            quote! {}
        } else {
            quote! {<#(#generic_idents),*>}
        };

        (generic_params, generic_idents)
    };
    let where_clause = item.sig.generics.where_clause;

    let visibility = item.vis;
    let body = item.block;

    let inputs = item.sig.inputs.into_iter().collect_vec();
    // NOTE(mithun): It is technically possible to support destructuring, but it's a niche use-case
    // for this particular application. Feel free to change it if you feel otherwise!
    assert!(
        inputs
            .iter()
            .flat_map(get_pat_type)
            .map(|t| match *t.pat {
                syn::Pat::Ident(_) => 0,
                syn::Pat::Wild(_) => 0,
                _ => 1,
            })
            .sum::<usize>()
            == 0,
        "your function signature uses destructuring; this macro only supports identifiers at present"
    );
    let (world_and_hooks, props) = inputs.split_at(2);

    // dirty check to ensure the first two arguments are (&mut world, &mut hooks)
    fn check_type_is_mut_ref(ty: &syn::Type, ident: &str) {
        if let syn::Type::Reference(tr) = ty {
            assert!(tr.mutability.is_some(), "expected {ty:?} to be a mutable reference");
            if let syn::Type::Path(path) = tr.elem.as_ref() {
                assert_eq!(path.path.segments.last().unwrap().ident, ident, "expected the last segment of the path to equal {ident}");
            } else {
                panic!("expected {tr:?} to be a mutable reference to a path");
            }
        } else {
            panic!("expected {ty:?} to be a reference to {ident}");
        }
    }
    check_type_is_mut_ref(get_pat_type(&world_and_hooks[0]).unwrap().ty.as_ref(), "World");
    check_type_is_mut_ref(get_pat_type(&world_and_hooks[1]).unwrap().ty.as_ref(), "Hooks");

    let struct_body = if props.is_empty() {
        quote! {
            ;
        }
    } else {
        quote! {
            {
                #(pub #props,)*
            }
        }
    };
    let (props_names_braced, struct_unpack) = if !props.is_empty() {
        let props_names = props.iter().flat_map(get_pat_type).map(|p| p.pat.as_ref()).collect_vec();
        let braced = quote! { { #(#props_names),* }  };
        (Some(braced.clone()), Some(quote! { let Self #braced = *self; }))
    } else {
        (None, None)
    };

    let el_block = with_el.then(|| {
        quote! {
            impl #generic_params #name #generic_idents #where_clause {
                pub fn el(#(#props),*) -> elements_element::Element {
                    use elements_element::ElementComponentExt;
                    Self #props_names_braced .el()
                }
            }
        }
    });

    quote! {
        #[derive(std::clone::Clone, std::fmt::Debug)]
        #(#attrs)*
        #visibility struct #name #generic_params #where_clause #struct_body
        impl #generic_params elements_element::ElementComponent for #name #generic_idents #where_clause {
            fn render(self: Box<Self>, #(#world_and_hooks),*) #ret {
                #struct_unpack
                #body
            }
        }
        #el_block
    }
}

#[cfg(test)]
mod test {
    use proc_macro2::TokenStream;
    use quote::quote;

    #[test]
    #[should_panic(expected = "assertion failed: mid <= self.len()")]
    fn test_invalid_base_args_1() {
        let input = quote! {
            pub fn ZeroArg(_: &mut NotAValidWorld) -> elements_element::Element {
                Element::new()
            }
        };

        let _ = super::do_derive_element_component(quote! {without_el}, input);
    }

    #[test]
    #[should_panic(
        expected = "assertion failed: `(left == right)`\n  left: `Ident(NotAValidHooks)`,\n right: `\"Hooks\"`: expected the last segment of the path to equal Hooks"
    )]
    fn test_invalid_base_args_2() {
        let input = quote! {
            pub fn ZeroArg(_: &mut World, _: &mut NotAValidHooks) -> elements_element::Element {
                Element::new()
            }
        };

        let _ = super::do_derive_element_component(quote! {without_el}, input);
    }

    #[test]
    #[should_panic(
        expected = "a `self` was specified in `fn ZeroArg (& self , _ : & mut elements_ecs :: World , _ : & mut elements_element :: Hooks) -> elements_element :: Element`; your function must be a free function for this macro to work"
    )]
    fn test_zero_arg_with_self_component() {
        let input = quote! {
            pub fn ZeroArg(&self, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                Element::new()
            }
        };

        let _ = super::do_derive_element_component(quote! {without_el}, input);
    }

    #[test]
    fn test_zero_arg_component() {
        let input = quote! {
            #[doc = "My cool comment"]
            pub fn ZeroArg(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks
            ) -> elements_element::Element {
                Element::new()
            }
        };

        let output = quote! {
            #[derive(std::clone::Clone, std::fmt::Debug)]
            #[doc = "My cool comment"]
            pub struct ZeroArg;
            impl elements_element::ElementComponent for ZeroArg {
                fn render(self: Box<Self>, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                    {
                        Element::new()
                    }
                }
            }
        };

        assert_eq!(super::do_derive_element_component(quote! {without_el}, input).to_string(), output.to_string());
    }

    #[test]
    fn test_zero_arg_with_el_component() {
        let input = quote! {
            #[doc = "My cool comment"]
            pub fn ZeroArg(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks
            ) -> elements_element::Element {
                Element::new()
            }
        };

        let output = quote! {
            #[derive(std::clone::Clone, std::fmt::Debug)]
            #[doc = "My cool comment"]
            pub struct ZeroArg;
            impl elements_element::ElementComponent for ZeroArg {
                fn render(self: Box<Self>, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                    {
                        Element::new()
                    }
                }
            }
            impl ZeroArg {
                pub fn el() -> elements_element::Element {
                    use elements_element::ElementComponentExt;
                    Self.el()
                }
            }
        };

        assert_eq!(super::do_derive_element_component(TokenStream::new(), input).to_string(), output.to_string());
    }

    #[test]
    fn test_single_arg_component() {
        let input = quote! {
            pub fn FancyText(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks,
                /// The message to display
                msg: String,
            ) -> elements_element::Element {
                Text::el(msg)
            }
        };

        let output = quote! {
            #[derive(std::clone::Clone, std::fmt::Debug)]
            pub struct FancyText {
                pub
                /// The message to display
                msg: String,
            }
            impl elements_element::ElementComponent for FancyText {
                fn render(self: Box<Self>, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                    let Self { msg } = *self;
                    {
                        Text::el(msg)
                    }
                }
            }
        };

        assert_eq!(super::do_derive_element_component(quote! {without_el}, input).to_string(), output.to_string());
    }

    #[test]
    #[should_panic(expected = "your function signature uses destructuring; this macro only supports identifiers at present")]
    fn test_single_arg_component_with_destructuring() {
        let input = quote! {
            pub fn FancyText(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks,
                /// The message to display
                Wrap(msg): Wrap,
            ) -> elements_element::Element {
                Text::el(msg)
            }
        };

        let _ = super::do_derive_element_component(quote! {without_el}, input);
    }

    #[test]
    fn test_choice_component_with_el() {
        let input = quote! {
            pub(crate) fn Choice(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks,
                msg: CowStr,
                choices: Vec<(CowStr, WorldCallback)>,
                post: WorldCallback,
            ) -> elements_element::Element {
                let buttons = choices
                    .into_iter()
                    .map(|(label, cb)| Button::new(label.to_string(), closure::closure!(std::clone::clone post, |w| {(cb)(w); (post)(w)})).el())
                    .collect_vec();

                FlowColumn::el([Text::el(msg), FlowRow(buttons).el()]).set(space_between_items(), 10.0)
            }
        };

        let output = quote! {
            #[derive(std::clone::Clone, std::fmt::Debug)]
            pub(crate) struct Choice {
                pub msg: CowStr,
                pub choices: Vec<(CowStr, WorldCallback)>,
                pub post: WorldCallback,
            }
            impl elements_element::ElementComponent for Choice {
                fn render(self: Box<Self>, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                    let Self { msg, choices, post } = *self;
                    {
                        let buttons = choices
                            .into_iter()
                            .map(|(label, cb)| Button::new(label.to_string(), closure::closure!(std::clone::clone post, |w| {(cb)(w); (post)(w)})).el())
                            .collect_vec();

                        FlowColumn::el([Text::el(msg), FlowRow(buttons).el()]).set(space_between_items(), 10.0)
                    }
                }
            }
            impl Choice {
                pub fn el(msg: CowStr, choices: Vec<(CowStr, WorldCallback)>, post: WorldCallback) -> elements_element::Element {
                    use elements_element::ElementComponentExt;
                    Self { msg, choices, post }.el()
                }
            }
        };

        assert_eq!(super::do_derive_element_component(TokenStream::new(), input).to_string(), output.to_string());
    }

    #[test]
    fn test_component_with_generics() {
        let input = quote! {
            pub(crate) fn GenericComponent<T1: Debug + 'static, T2>(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks,
                _a: T1,
                _b: T2,
            ) -> elements_element::Element
            where T2: Debug + 'static {
                Element::new()
            }
        };

        let output = quote! {
            #[derive(std::clone::Clone, std::fmt::Debug)]
            pub(crate) struct GenericComponent<T1: Debug + 'static, T2> where T2: Debug + 'static {
                pub _a: T1,
                pub _b: T2,
            }
            impl<T1: Debug + 'static, T2> elements_element::ElementComponent for GenericComponent<T1, T2> where T2: Debug + 'static {
                fn render(self: Box<Self>, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                    let Self { _a, _b } = *self;
                    {
                        Element::new()
                    }
                }
            }
        };

        assert_eq!(super::do_derive_element_component(quote! {without_el}, input).to_string(), output.to_string());
    }

    #[test]
    fn test_component_with_generics_and_el() {
        let input = quote! {
            pub(crate) fn GenericComponent<T1: Debug + 'static, T2>(
                _: &mut elements_ecs::World,
                _: &mut elements_element::Hooks,
                a: T1,
                b: T2,
            ) -> elements_element::Element
            where T2: Debug + 'static {
                Element::new()
            }
        };

        let output = quote! {
            #[derive(std::clone::Clone, std::fmt::Debug)]
            pub(crate) struct GenericComponent<T1: Debug + 'static, T2> where T2: Debug + 'static {
                pub a: T1,
                pub b: T2,
            }
            impl<T1: Debug + 'static, T2> elements_element::ElementComponent for GenericComponent<T1, T2> where T2: Debug + 'static {
                fn render(self: Box<Self>, _: &mut elements_ecs::World, _: &mut elements_element::Hooks) -> elements_element::Element {
                    let Self { a, b } = *self;
                    {
                        Element::new()
                    }
                }
            }
            impl<T1: Debug + 'static, T2> GenericComponent<T1, T2> where T2: Debug + 'static {
                pub fn el(a: T1, b: T2) -> elements_element::Element {
                    use elements_element::ElementComponentExt;
                    Self { a, b }.el()
                }
            }
        };

        assert_eq!(super::do_derive_element_component(TokenStream::new(), input).to_string(), output.to_string());
    }
}
