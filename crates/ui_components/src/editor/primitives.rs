use std::str::FromStr;
use std::{fmt::Debug, time::Duration};

use ambient_cb::{cb, Cb};
use ambient_element::{define_el_function_for_vec_element_newtype, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::layout::margin_right;
use convert_case::{Case, Casing};
use glam::{Vec2, Vec3, Vec4};
use itertools::Itertools;

use crate::button::{Button, ButtonStyle};
use crate::default_theme::STREET;
use crate::layout::{FlowColumn, FlowRow};
use crate::text::{FontAwesomeIcon, Text};
use crate::use_focus_for_instance_id;

use super::{ChangeCb, Editor, EditorOpts, TextEditor};

#[derive(Debug, Clone)]
pub struct ParseableInput<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> {
    pub value: T,
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}
impl<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> ParseableInput<T> {
    pub fn new(value: T, on_change: impl Fn(T) + Sync + Send + 'static) -> Self {
        Self { value, on_change: cb(on_change) }
    }
}
impl<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> ElementComponent for ParseableInput<T> {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        let (text_id, set_text_id) = hooks.use_state(String::new());
        let (focused, _) = use_focus_for_instance_id(hooks, text_id);
        let (text, set_text) = hooks.use_state(None);
        if focused && text.is_none() {
            set_text(Some(value.to_string()));
        } else if !focused && text.is_some() {
            set_text(None);
        }
        TextEditor::new(
            text.unwrap_or_else(|| value.to_string()),
            cb(move |text| {
                if let Ok(value) = text.parse::<T>() {
                    on_change.0(value);
                }
                set_text(Some(text));
            }),
        )
        .el()
        .on_spawned(move |_, _, id| set_text_id(id.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct CustomParseInput<T> {
    pub value: T,
    pub parse: Cb<dyn Fn(&str) -> Option<T> + Sync + Send>,
    pub to_string: Cb<dyn Fn(&T) -> String + Sync + Send>,
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}

impl<T: Debug + Clone + Sync + Send + 'static> ElementComponent for CustomParseInput<T> {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { value, on_change, parse, to_string } = *self;

        let (text_id, set_text_id) = hooks.use_state(String::new());
        let (focused, _) = use_focus_for_instance_id(hooks, text_id);
        let (text, set_text) = hooks.use_state(None);
        if focused && text.is_none() {
            set_text(Some(to_string(&value)));
        } else if !focused && text.is_some() {
            set_text(None);
        }
        TextEditor::new(
            text.unwrap_or_else(|| to_string(&value)),
            cb(move |text| {
                if let Some(value) = parse(&text) {
                    on_change.0(value);
                }
                set_text(Some(text));
            }),
        )
        .el()
        .on_spawned(move |_, _, id| set_text_id(id.to_string()))
    }
}

pub type F32Input = ParseableInput<f32>;
pub type I32Input = ParseableInput<i32>;
pub type U32Input = ParseableInput<u32>;
pub type U64Input = ParseableInput<u64>;
pub type UsizeInput = ParseableInput<usize>;

impl Editor for Duration {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        CustomParseInput {
            value: self,
            on_change,
            parse: cb(|v| v.parse::<f32>().ok().map(Duration::from_secs_f32)),
            to_string: cb(|v| format!("{:.3}", v.as_secs_f32())),
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element
    where
        Self: Sized,
    {
        Text::el(format!("{}", self.as_secs_f32()))
    }
}

impl Editor for f32 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        F32Input { value: self, on_change }.el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for i32 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        I32Input { value: self, on_change }.el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for u32 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        U32Input { value: self, on_change }.el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for u64 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        U64Input { value: self, on_change }.el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for usize {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        UsizeInput { value: self, on_change }.el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}

#[derive(Clone, Debug)]
pub struct Checkbox {
    pub value: bool,
    pub on_change: Cb<dyn Fn(bool) + Sync + Send>,
}
impl Checkbox {
    pub fn new(value: bool, on_change: impl Fn(bool) + Sync + Send + 'static) -> Self {
        Self { value, on_change: cb(on_change) }
    }
}
impl ElementComponent for Checkbox {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Checkbox { value, on_change } = *self;
        Button::new(FontAwesomeIcon::el(if value { 0xf14a } else { 0xf0c8 }, false), move |_| on_change.0(!value))
            .style(ButtonStyle::Flat)
            .el()
    }
}

impl Editor for bool {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        Checkbox { value: self, on_change }.el()
    }

    fn view(self, _: EditorOpts) -> Element {
        if self {
            FontAwesomeIcon::el(0xf14a, false)
        } else {
            FontAwesomeIcon::el(0xf0c8, false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditorRow {
    title: String,
    editor: Element,
}
impl EditorRow {
    pub fn el(title: impl Into<String>, editor: Element) -> Element {
        let title: String = title.into();
        EditorRow { title: title.to_case(Case::Title), editor }.el()
    }
}
impl ElementComponent for EditorRow {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        let Self { title, editor } = *self;
        FlowRow(vec![Text::el(title).set(margin_right(), STREET), editor]).el()
    }
}

#[derive(Debug, Clone)]
pub struct EditorColumn(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(EditorColumn);
impl ElementComponent for EditorColumn {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        FlowColumn(self.0).el()
    }
}

impl Editor for Vec2 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor { value: self.to_array(), on_change: cb(move |v| (on_change)(Self::from(v))), field_names: Some(&["X", "Y"]) }.el()
    }
}

impl Editor for Vec3 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor { value: self.to_array(), on_change: cb(move |v| (on_change)(Self::from(v))), field_names: Some(&["X", "Y", "Z"]) }.el()
    }
}

impl Editor for Vec4 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor { value: self.to_array(), on_change: cb(move |v| (on_change)(Self::from(v))), field_names: Some(&["X", "Y", "Z", "W"]) }
            .el()
    }
}

#[derive(Debug, Clone)]
pub struct ArrayEditor<const C: usize, T> {
    pub value: [T; C],
    pub field_names: Option<&'static [&'static str; C]>,
    pub on_change: Cb<dyn Fn([T; C]) + Sync + Send>,
}

impl<const C: usize, T: 'static + Clone + Debug + Editor + Send + Sync> ElementComponent for ArrayEditor<C, T> {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self { value, on_change, field_names } = *self;

        if let Some(field_names) = field_names {
            EditorColumn(
                value
                    .iter()
                    .enumerate()
                    .zip_eq(field_names)
                    .map(|((i, v), &name)| {
                        let value = value.clone();
                        let on_change = on_change.clone();
                        EditorRow::el(
                            name,
                            v.clone().editor(
                                cb(move |v| {
                                    let mut value = value.clone();
                                    value[i] = v;
                                    on_change(value)
                                }),
                                Default::default(),
                            ),
                        )
                    })
                    .collect_vec(),
            )
            .el()
        } else {
            EditorColumn(
                value
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let value = value.clone();
                        let on_change = on_change.clone();
                        v.clone().editor(
                            cb(move |v| {
                                let mut value = value.clone();
                                value[i] = v;
                                on_change(value)
                            }),
                            Default::default(),
                        )
                    })
                    .collect_vec(),
            )
            .el()
        }
    }
}
impl<const C: usize, T: 'static + Clone + Debug + Editor + Send + Sync> Editor for [T; C] {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor { value: self, on_change, field_names: None }.el()
    }
}
