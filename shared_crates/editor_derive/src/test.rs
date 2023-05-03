use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;
use quote::quote;

use crate::do_derive_element_editor;

fn test_base(body: TokenStream) -> TokenStream {
    quote! {

        #[derive(Clone, Debug)]
        pub struct TestEditor {
            pub value: Test,
            pub on_change: Option<ambient_ui_native::cb::Cb<dyn Fn(Test) + ::std::marker::Sync + ::std::marker::Send>>,
            pub opts: ambient_ui_native::ui::editor::EditorOpts,
        }
        #[automatically_derived]
        impl ambient_ui_native::element::ElementComponent for TestEditor {
            fn render(self: Box<Self>, hooks: &mut ambient_ui_native::element::Hooks) -> ambient_ui_native::element::Element {
                use ambient_ui_native::element::{Element, ElementComponentExt};
                use ambient_ui_native::ui::{editor::{Editor, EditorRow, EditorColumn, Slider, IntegerSlider}, select::{ListSelect, DropdownSelect}, layout::{FlowRow, FlowColumn}, text::Text};
                use ambient_ui_native::layout::{margin, fit_horizontal_parent};
                let Self { value, on_change, opts } = *self;
                #body
            }
        }

        impl ambient_ui_native::ui::editor::Editor for Test {
            fn editor(self, on_change: ambient_ui_native::ui::editor::ChangeCb<Self>, opts: ambient_ui_native::ui::editor::EditorOpts) -> ambient_ui_native::element::Element {
                TestEditor { value: self, on_change: Some(on_change), opts }.into()
            }

            fn view(self, opts: ambient_ui_native::ui::editor::EditorOpts) -> ambient_ui_native::element::Element {
                TestEditor { value: self, on_change: None, opts }.into()
            }
        }

    }
}

#[test]
fn test_struct() {
    let input = quote! {
        struct Test {
            my_f32_field: f32,
            my_option: Option<bool>
        }
    };
    let output: TokenStream = do_derive_element_editor(input);
    assert_eq!(
        output.to_string(),
        test_base(quote! {
            let Test { my_f32_field, my_option } = value;
            EditorColumn(vec![

                EditorRow::el(
                    "my_f32_field",
                    <f32 as ambient_ui_native::ui::editor::Editor>::edit_or_view(my_f32_field.clone(), on_change.clone().map(|on_change| -> ambient_ui_native::cb::Cb<dyn Fn(f32) + ::std::marker::Sync + ::std::marker::Send> {
                        ambient_ui_native::cb::cb({
                            let my_option = my_option.clone();
                            move |v| {
                                on_change.0(Test { my_f32_field: v, my_option: my_option.clone() });
                            }
                        })
                    }), Default::default())
                ),

                EditorRow::el(
                    "my_option",
                    <Option::<bool> as ambient_ui_native::ui::editor::Editor>::edit_or_view(my_option.clone(), on_change.clone().map(|on_change| -> ambient_ui_native::cb::Cb<dyn Fn(Option<bool>) + ::std::marker::Sync + ::std::marker::Send> {
                        ambient_ui_native::cb::cb({
                            let my_f32_field = my_f32_field.clone();
                            move |v| {
                                on_change.0(Test { my_f32_field: my_f32_field.clone(), my_option: v });
                            }
                        })
                    }), Default::default())
                ),

            ]).el()

        })
        .to_string()
    );
}

#[test]
fn test_enum() {
    let create_variant_fragment = quote! {
        fn create_variant(variant_index: usize) -> Test {
            match variant_index {
                0usize => Test::First,
                1usize => Test::Second {
                    testy: Default::default(),
                },
                2usize => Test::Third(Default::default()),
                _ => unreachable!()
            }
        }
    };

    let container_contents_fragment = quote! {
        vec![
            if opts.enum_can_change_type {
                if let Some(on_change) = on_change.clone() {
                    ListSelect {
                        value: match &value {
                            Test::First => 0usize,
                            Test::Second { .. } => 1usize,
                            Test::Third(_) => 2usize,
                        },
                        on_change: ambient_ui_native::cb::cb(
                            move |index| on_change.0(create_variant(index))
                        ),
                        items: vec![
                            Text::el("First"),
                            Text::el("Second"),
                            Text::el("Third"),
                        ],
                        inline: true
                    }.el()
                } else {
                    ambient_ui_native::element::Element::new()
                }
            } else {
                ambient_ui_native::element::Element::new()
            },

            match value {
                Test::First => ambient_ui_native::element::Element::new(),
                Test::Second { testy } => EditorColumn(vec![

                    EditorRow::el(
                        "testy",
                        <f32 as ambient_ui_native::ui::editor::Editor>::edit_or_view(testy.clone(), on_change.clone().map(|on_change| -> ambient_ui_native::cb::Cb<dyn Fn(f32) + ::std::marker::Sync + ::std::marker::Send> {
                            ambient_ui_native::cb::cb({
                                move |v| {
                                    on_change.0(Test::Second { testy: v });
                                }
                            }
                        )}), Default::default())
                    ),

                ]).el(),

                Test::Third(field_0) => EditorColumn(vec![

                    EditorRow::el(
                        "",
                        <f32 as ambient_ui_native::ui::editor::Editor>::edit_or_view(field_0.clone(), on_change.clone().map(|on_change| -> ambient_ui_native::cb::Cb<dyn Fn(f32) + ::std::marker::Sync + ::std::marker::Send> {
                            ambient_ui_native::cb::cb({
                                move |v| {
                                    on_change.0(Test::Third(v));
                                }
                            })
                        }), Default::default())
                    ),

                ]).el(),
            }
        ]
    };

    let input = quote! {
        enum Test {
            First,
            Second { testy: f32 },
            Third(f32)
        }
    };

    assert_eq!(
        do_derive_element_editor(input.clone()).to_string(),
        test_base(quote! {
            #create_variant_fragment
            FlowColumn(#container_contents_fragment).el()

        })
        .to_string()
    );

    assert_eq!(
        do_derive_element_editor(quote! { #[editor(inline)] #input }).to_string(),
        test_base(quote! {
            #create_variant_fragment
            FlowRow(#container_contents_fragment).el()

        })
        .to_string()
    );
}

#[test]
fn test_enum_inline() {
    let input = quote! {
        enum Test {
            #[editor(inline = "Hello {testy}")]
            First { testy: f32 },
        }
    };
    let output: TokenStream = do_derive_element_editor(input);
    assert_eq!(
        output.to_string(),
        test_base(quote! {
            fn create_variant(variant_index: usize) -> Test {
                match variant_index {
                    0usize => Test::First {
                        testy: Default::default(),
                    },
                    _ => unreachable!()
                }
            }
            let field_editors = match value {
                Test::First { testy } => FlowRow(vec![

                    Text::el("Hello "),
                    <f32 as ambient_ui_native::ui::editor::Editor>::edit_or_view(testy.clone(), on_change.clone().map(|on_change| -> ambient_ui_native::cb::Cb<dyn Fn(f32) + ::std::marker::Sync + ::std::marker::Send> {
                        ambient_ui_native::cb::cb({
                            move |v| {
                                on_change.0(Test::First { testy: v });
                            }
                        })
                    }), Default::default())
                    .set(margin(), vec4(0., 0., 0., 5.))

                ]).el(),
            };
            if opts.enum_can_change_type {
                if let Some(on_change) = on_change {
                    DropdownSelect {
                        content: field_editors,
                        on_select: ambient_ui_native::cb::cb(
                            move |index| on_change.0(create_variant(index))
                        ),
                        items: vec![
                            Test::view(create_variant(0usize), Default::default()),
                        ],
                        inline: true
                    }.el()
                } else {
                    field_editors
                }
            } else {
                field_editors
            }

        })
        .to_string()
    );
}

#[test]
fn test_custom_editor() {
    let input = quote! {
        struct Test {
            #[editor(editor="test_editor")]
            my_f32_field: f32,
        }
    };
    let output: TokenStream = do_derive_element_editor(input);
    assert_eq!(
        output.to_string(),
        test_base(quote! {
            let Test { my_f32_field } = value;
            EditorColumn(vec![

                EditorRow::el(
                    "my_f32_field",
                    test_editor(my_f32_field.clone(), on_change.clone().map(|on_change| -> ambient_ui_native::cb::Cb<dyn Fn(f32) + ::std::marker::Sync + ::std::marker::Send> {
                        ambient_ui_native::cb::cb({
                            move |v| {
                                on_change.0(Test { my_f32_field: v });
                            }
                        })}), Default::default())
                ),

            ]).el()

        })
        .to_string()
    );
}
