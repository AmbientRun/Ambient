extern crate proc_macro;
extern crate quote;
use std::collections::HashMap;

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, AngleBracketedGenericArguments, Data, DataEnum, DataStruct, DeriveInput, Field,
    Fields, FieldsNamed, FieldsUnnamed, Lit, LitStr, Path, PathArguments, Token, Type, TypePath, Variant,
};

mod attributes;
mod inline_string;
#[cfg(test)]
mod test;
use attributes::*;
use inline_string::*;

#[proc_macro_derive(ElementEditor, attributes(editor, editor_inline, prompt))]
pub fn derive_element_editor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_derive_element_editor(input.into()).into()
}

fn do_derive_element_editor(input: TokenStream) -> TokenStream {
    let input = match syn::parse2::<DeriveInput>(input) {
        Ok(syntax_tree) => syntax_tree,
        Err(err) => return err.to_compile_error(),
    };

    // let found_crate = crate_name("elements");
    // let _root = match found_crate {
    //     Ok(FoundCrate::Itself) => quote!(crate),
    //     Ok(FoundCrate::Name(name)) => {
    //         let ident = Ident::new(&name, Span::call_site());
    //         quote!( #ident )
    //     }
    //     _ => quote!(crate),
    // };

    let ui_crate = match crate_name("ambient_ui") {
        Ok(FoundCrate::Itself) => Ident::new("crate", Span::call_site()),
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
        Err(err) => panic!("Missing crate `ambient_ui`. {err:?}"),
    };

    let attributes = EditorAttrs::parse(&input.attrs);
    let editor_name = Ident::new(&format!("{}Editor", input.ident), input.ident.span());
    let type_name = input.ident.clone();

    let editor_impl = if let Some(title) = attributes.prompt.as_ref() {
        let title = title.clone().unwrap_or_else(|| LitStr::new(&type_name.to_string(), Span::call_site()));

        quote! {
            impl #ui_crate::Editor for #type_name {
                fn editor(self, on_change: #ui_crate::ChangeCb<Self>, opts: #ui_crate::EditorOpts) -> #ui_crate::element::Element {
                    let editor = #ui_crate::cb(|value, on_change, opts| #editor_name { value, on_change, opts }.into());
                    #ui_crate::OffscreenEditor { title: #title.into(), opts, value: self, on_confirm: Some(on_change), editor }.into()
                }

                fn view(self, opts: #ui_crate::EditorOpts) -> #ui_crate::element::Element {
                    let editor = #ui_crate::cb(|value, on_change, opts| #editor_name { value, on_change, opts }.into());
                    #ui_crate::OffscreenEditor { title: #title.into(), opts, value: self, on_confirm: None, editor }.into()
                }
            }
        }
    } else {
        quote! {

            impl #ui_crate::Editor for #type_name {
                fn editor(self, on_change: #ui_crate::ChangeCb<Self>, opts: #ui_crate::EditorOpts) -> #ui_crate::element::Element {
                    #editor_name { value: self, on_change: Some(on_change), opts }.into()
                }

                fn view(self, opts: #ui_crate::EditorOpts) -> #ui_crate::element::Element {
                    #editor_name { value: self, on_change: None, opts }.into()
                }
            }
        }
    };

    let body = body_to_tokens(&ui_crate, attributes, type_name.clone(), input);

    quote! {

        #[derive(Clone, Debug)]
        pub struct #editor_name {
            pub value: #type_name,
            pub on_change: Option<#ui_crate::Cb<dyn Fn(#type_name) + ::std::marker::Sync + ::std::marker::Send>>,
            pub opts: #ui_crate::EditorOpts,
        }

        #[automatically_derived]
        impl #ui_crate::element::ElementComponent for #editor_name {
            fn render(self: Box<Self>, hooks: &mut #ui_crate::element::Hooks) -> #ui_crate::element::Element {
                use #ui_crate::element::{Element, ElementComponentExt};
                use #ui_crate::{Editor, EditorRow, EditorColumn, Slider, IntegerSlider, ListSelect, DropdownSelect, FlowRow, FlowColumn, Text, layout::{margin, Borders, fit_horizontal, Fit}};
                let Self { value, on_change, opts } = *self;
                #body
            }
        }

        #editor_impl
    }
}

fn body_to_tokens(ui_crate: &Ident, attrs: EditorAttrs, type_name: Ident, input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => {
            let inline = inline_text(attrs.inline);
            let field_editors = fields_editor(ui_crate, quote! { #type_name }, &fields.named, inline, false);
            let field_destruct = fields.named.iter().map(|field| {
                let field_ident = &field.ident;
                quote! { #field_ident }
            });
            quote! {
                let #type_name { #(#field_destruct),* } = value;
                #field_editors
            }
        }
        Data::Struct(DataStruct { fields: Fields::Unnamed(fields), .. }) => {
            let inline = inline_text(attrs.inline);
            let field_editors = fields_editor(ui_crate, quote! { #type_name }, &fields.unnamed, inline, true);
            let field_destruct = fields.unnamed.iter().enumerate().map(|(i, _field)| {
                let name = Ident::new(&format!("field_{i}"), Span::call_site());
                quote! { #name }
            });
            quote! {
                let #type_name(#(#field_destruct),*) = value;
                #field_editors
            }
        }
        Data::Enum(DataEnum { variants, .. }) => enum_to_tokens(ui_crate, type_name, variants, attrs),
        _ => panic!("this derive macro only works on structs and enums with named fields"),
    }
}

fn create_enum_variant_constructor(type_name: Ident, variants: &Punctuated<Variant, Comma>) -> TokenStream {
    let variants_constructors = variants.iter().enumerate().map(|(i, variant)| {
        let variant_ident = &variant.ident;
        match &variant.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let field_ctors = named.iter().map(|field| {
                    let field_ident = &field.ident;
                    quote! { #field_ident: Default::default(), }
                });
                quote! { #i => #type_name::#variant_ident {
                    #(#field_ctors)*
                }, }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let field_ctors = unnamed.iter().map(|_| {
                    quote! { Default::default() }
                });
                quote! { #i => #type_name::#variant_ident(
                    #(#field_ctors),*
                ), }
            }
            Fields::Unit => quote! { #i => #type_name::#variant_ident, },
        }
    });
    quote! {
        fn create_variant(variant_index: usize) -> #type_name {
            match variant_index {
                #(#variants_constructors)*
                _ => unreachable!()
            }
        }
    }
}

fn enum_to_tokens(ui_crate: &Ident, type_name: Ident, variants: Punctuated<Variant, Comma>, attrs: EditorAttrs) -> TokenStream {
    let value_matches = variants.iter().enumerate().map(|(i, variant)| {
        let variant_ident = &variant.ident;
        match variant.fields {
            Fields::Named(_) => quote! { #type_name::#variant_ident { .. } => #i, },
            Fields::Unnamed(_) => quote! { #type_name::#variant_ident(_) => #i, },
            Fields::Unit => quote! { #type_name::#variant_ident => #i, },
        }
    });

    let variant_constructor = create_enum_variant_constructor(type_name.clone(), &variants);
    let mut has_inline = false;
    let field_editors = variants.iter().enumerate().map(|(_, variant)| {
        let inline = inline_text(EditorAttrs::parse(&variant.attrs).inline);
        has_inline = has_inline || inline.is_some();
        let variant_ident = &variant.ident;

        match &variant.fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let editors = fields_editor(ui_crate, quote! { #type_name::#variant_ident }, named, inline, false);
                let field_destruct = named.iter().map(|field| {
                    let field_ident = &field.ident;
                    quote! { #field_ident }
                });
                quote! { #type_name::#variant_ident { #(#field_destruct),* } => #editors, }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let editors = fields_editor(ui_crate, quote! { #type_name::#variant_ident }, unnamed, inline, true);
                let field_destruct = unnamed.iter().enumerate().map(|(i, _field)| {
                    let name = Ident::new(&format!("field_{i}"), Span::call_site());
                    quote! { #name }
                });

                quote! { #type_name::#variant_ident(#(#field_destruct),*) => #editors, }
            }
            Fields::Unit => {
                let editor = if let Some(inline) = inline {
                    quote! { #ui_crate::Text::el(#inline) }
                } else {
                    quote! { #ui_crate::element::Element::new() }
                };
                quote! { #type_name::#variant_ident => #editor, }
            }
        }
    });
    let field_editors = quote! {
        match value {
            #(#field_editors)*
        }
    };
    let on_change_cb = quote! {
        #ui_crate::cb(move |index| on_change.0(create_variant(index)))
    };
    let inner = if has_inline {
        let variants_readonly_items = variants.iter().enumerate().map(|(i, _)| {
            quote! {
                #type_name::view(create_variant(#i), Default::default()),
            }
        });
        quote! {
            let field_editors = #field_editors;
            if opts.enum_can_change_type {
                if let Some(on_change) = on_change {
                    #ui_crate::DropdownSelect {
                        content: field_editors,
                        on_select: #on_change_cb,
                        items: vec![
                            #(#variants_readonly_items)*
                        ],
                        inline: true
                    }.el()
                } else {
                    field_editors
                }
            } else {
                field_editors
            }
        }
    } else {
        let variants_name_items = variants.iter().enumerate().map(|(_, variant)| {
            let name = variant.ident.to_string();
            quote! {
                Text::el(#name),
            }
        });
        let variant_changer = quote! {
            if opts.enum_can_change_type {
                if let Some(on_change) = on_change.clone() {
                    ListSelect {
                        value: match &value {
                            #(#value_matches)*
                        },
                        on_change: #on_change_cb,
                        items: vec![
                            #(#variants_name_items)*
                        ],
                        inline: true
                    }.el()
                } else {
                    #ui_crate::element::Element::new()
                }
            } else {
                #ui_crate::element::Element::new()
            },
        };

        let container_type = if attrs.inline == Some(InlineType::InlineRow) {
            quote! { FlowRow }
        } else {
            quote! { FlowColumn }
        };
        quote! {
            #container_type(vec![
                #variant_changer
                #field_editors
            ]).el()
        }
    };
    quote! {
        #variant_constructor
        #inner
    }
}

fn fields_editor(
    ui_crate: &Ident,
    type_ctor: TokenStream,
    fields: &Punctuated<Field, Comma>,
    inline: Option<String>,
    unnamed: bool,
) -> TokenStream {
    let field_editors = fields.iter().enumerate().filter_map(|(i, field)| {
        let field_name = field.ident.clone().unwrap_or_else(|| Ident::new(&format!("field_{i}"), field.span()));

        let field_ty = field.ty.clone();
        let field_ty_colon = type_with_colon(&field_ty);

        let attrs = EditorAttrs::parse(&field.attrs);

        if attrs.hidden {
            return None;
        }

        let other_fields_cloned = fields.iter().filter(|f| f.ident != field.ident).map(|f| {
            let f_ident = &f.ident;
            quote! { let #f_ident = #f_ident.clone(); }
        });
        let fields_expanded = fields.iter().enumerate().map(|(inner_i, f)| {
            let f_ident = &f.ident;

            if unnamed {
                if inner_i != i {
                    quote! { #f_ident.clone() }
                } else {
                    quote! { v }
                }
            } else if f.ident != field.ident {
                quote! { #f_ident: #f_ident.clone() }
            } else {
                quote! { #f_ident: v }
            }
        });
        let fields_expanded = if unnamed {
            quote! { ( #(#fields_expanded),* ) }
        } else {
            quote! { { #(#fields_expanded),* } }
        };

        let on_change_cb = quote! {
            on_change.clone().map(|on_change| -> #ui_crate::Cb<dyn Fn(#field_ty) + ::std::marker::Sync + ::std::marker::Send> {
                #ui_crate::cb({
                    #(#other_fields_cloned)*
                    move |v| {
                        on_change.0(#type_ctor #fields_expanded);
                    }
                })
            })
        };

        let editor = if attrs.slider {
            let field_ty_name = field_ty_colon.to_token_stream().to_string();
            let is_float_slider = match &field_ty_name as &str {
                "f32" => true,
                "i32" => false,
                _ => panic!("Slider is not supported for {field_ty_name}"),
            };
            let (slider, round) = if is_float_slider {
                let round = attrs.round;
                (quote! { Slider }, quote! { round: Some(#round), })
            } else {
                (quote! { IntegerSlider }, TokenStream::new())
            };

            let value_min = attrs.value_min;
            let value_max = attrs.value_max;
            let logarithmic = attrs.logarithmic;

            quote! {
                #slider {
                    value: #field_name.clone(),
                    on_change: #on_change_cb,
                    min: #value_min,
                    max: #value_max,
                    width: 200.,
                    logarithmic: #logarithmic,
                    suffix: None,
                    #round
                }.el()
            }
        } else if let Some(Lit::Str(custom_editor)) = attrs.editor {
            let custom_editor: Ident = custom_editor.parse().unwrap();
            quote! {
                #custom_editor(#field_name.clone(), #on_change_cb, Default::default())
            }
        } else if let Some(title) = attrs.prompt {
            let title = title.unwrap_or_else(|| LitStr::new(&field_name.to_string(), field.span()));

            quote! {
                {
                    let editor = #ui_crate::cb( <#field_ty_colon as ambient_ui::Editor>::edit_or_view );
                    #ui_crate::OffscreenEditor { title: #title.into(), value: #field_name.clone(), on_confirm: #on_change_cb, opts: Default::default(), editor }.into()
                }
            }
        } else {
            quote! {
                <#field_ty_colon as ambient_ui::Editor>::edit_or_view(#field_name.clone(), #on_change_cb, Default::default())
            }
        };

        Some((field_name.to_string(), editor))
    });

    if let Some(inline) = inline {
        let fields = field_editors.collect::<HashMap<_, _>>();
        let mut rows = parse_inline_string(&inline)
            .into_iter()
            .map(|row| {
                let field_editors = row.into_iter().enumerate().map(|(i, item)| {
                    let margin = if i > 0 {
                        quote! { .set(margin(), Borders::left(5.)) }
                    } else {
                        quote! {}
                    };
                    match item {
                        Inline::Text(text) => quote! { Text::el(#text)#margin },
                        Inline::Field(field) => {
                            let field_editor = fields.get(&field).unwrap_or_else(|| panic!("No such field: {field:?}")).clone();
                            quote! { #field_editor #margin }
                        }
                    }
                });
                quote! {
                    FlowRow(vec![
                        #(#field_editors),*
                    ]).el()
                }
            })
            .collect::<Vec<_>>();
        if rows.len() == 1 {
            rows.pop().unwrap()
        } else {
            quote! {
                FlowColumn(vec![
                    #(#rows .set(fit_horizontal(), Fit::Parent)),*
                ]).el()
                    .set(fit_horizontal(), Fit::Parent)
            }
        }
    } else {
        let field_editors = field_editors.map(|(field_text_name, editor)| {
            let field_text_name = if unnamed { "".to_string() } else { field_text_name };
            quote! {
                EditorRow::el(#field_text_name, #editor),
            }
        });
        quote! {
            EditorColumn(vec![
                #(#field_editors)*
            ]).el()
        }
    }
}

fn type_with_colon(ty: &Type) -> Type {
    let mut ty = ty.clone();
    if let Type::Path(TypePath { path: Path { segments, .. }, .. }) = &mut ty {
        if let Some(first) = segments.first_mut() {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { colon2_token, .. }) = &mut first.arguments {
                *colon2_token = Some(Token![::](Span::call_site()));
            }
        }
    }
    ty
}
