use proc_macro2::Span;
use quote::ToTokens;
use syn::{Attribute, Lit, LitBool, LitFloat, LitInt, LitStr, Meta, MetaList, MetaNameValue, NestedMeta};

#[derive(Debug, PartialEq, Eq)]
pub enum InlineType {
    InlineRow,
    InlineText(String),
}

pub fn inline_text(ty: Option<InlineType>) -> Option<String> {
    match ty {
        Some(InlineType::InlineText(s)) => Some(s),
        _ => None,
    }
}

#[derive(Debug, Default)]
pub struct OuterAttributes {
    pub prompt: bool,
    pub name: Option<LitStr>,
    pub inline: Option<InlineType>,
}

fn get_editor_meta(attrs: &[Attribute]) -> impl Iterator<Item = Result<Vec<NestedMeta>, syn::Error>> + '_ {
    attrs.iter().flat_map(|attr| match attr.parse_meta() {
        Ok(Meta::List(MetaList { path, nested, .. })) if path.is_ident("editor") => Some(Ok(nested.into_iter().collect())),
        Err(e) => Some(Err(e)),
        _ => None,
    })
}

pub struct EditorAttrs {
    pub prompt: Option<Option<LitStr>>,
    pub inline: Option<InlineType>,
    pub hidden: bool,
    pub slider: bool,
    pub value_min: Lit,
    pub value_max: Lit,
    pub logarithmic: Lit,
    pub round: Lit,
    pub editor: Option<Lit>,
}

impl EditorAttrs {
    pub fn parse(attrs: &[Attribute]) -> Self {
        let mut res = Self {
            inline: None,
            hidden: false,
            slider: false,
            value_min: Lit::Float(LitFloat::new("0.", Span::call_site())),
            value_max: Lit::Float(LitFloat::new("1.", Span::call_site())),
            logarithmic: Lit::Bool(LitBool::new(false, Span::call_site())),
            round: Lit::Int(LitInt::new("2", Span::call_site())),
            editor: None,
            prompt: None,
        };
        for attr in get_editor_meta(attrs) {
            let nested = attr.unwrap();

            for n in nested.into_iter() {
                match n {
                    NestedMeta::Meta(Meta::Path(path)) => {
                        let attr = &path.to_token_stream().to_string() as &str;
                        match attr {
                            "inline" => res.inline = Some(InlineType::InlineRow),
                            "hidden" => res.hidden = true,
                            "slider" => res.slider = true,
                            "prompt" => res.prompt = Some(None),
                            "logarithmic" => res.logarithmic = Lit::Bool(LitBool::new(true, Span::call_site())),
                            _ => panic!("Unrecognized attribute: {attr}"),
                        }
                    }
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                        let attr = &path.to_token_stream().to_string() as &str;
                        match attr {
                            "logarithmic" => res.logarithmic = lit,
                            "prompt" => match lit {
                                Lit::Str(lit) => res.prompt = Some(Some(lit)),
                                _ => panic!("Expected string literal"),
                            },
                            "min" => res.value_min = lit,
                            "max" => res.value_max = lit,
                            "round" => res.round = lit,
                            "inline" => {
                                res.inline = match lit {
                                    Lit::Str(str) => Some(InlineType::InlineText(str.value())),
                                    _ => panic!("Expected inline attribute to be a string"),
                                }
                            }
                            "editor" => res.editor = Some(lit),
                            _ => panic!("Unrecognized attribute: {attr} = {lit:?}"),
                        }
                    }
                    _ => {}
                }
            }
        }
        res
    }
}
