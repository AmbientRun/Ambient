use std::{fmt::Debug, str::FromStr, time::Duration};

use ambient_cb::{cb, Cb};
use ambient_element::{
    define_el_function_for_vec_element_newtype, to_owned, Element, ElementComponent,
    ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::layout::components::margin;
use convert_case::{Case, Casing};
use glam::{vec4, Vec2, Vec3, Vec4};
use itertools::Itertools;

use super::{ChangeCb, Editor, EditorOpts, TextEditor};
use crate::{
    button::{Button, ButtonStyle},
    default_theme::STREET,
    layout::{FlowColumn, FlowRow},
    text::{FontAwesomeIcon, Text},
    use_focus_for_instance_id,
};

#[derive(Debug, Clone)]
/// An editor for a value that can be parsed from a string.
pub struct ParseableInput<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> {
    /// The current value.
    pub value: T,
    /// Callback for when the value changes.
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}
impl<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> ParseableInput<T> {
    /// Create a new `ParseableInput` with the given value and callback.
    pub fn new(value: T, on_change: impl Fn(T) + Sync + Send + 'static) -> Self {
        Self {
            value,
            on_change: cb(on_change),
        }
    }
}
impl<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> ElementComponent
    for ParseableInput<T>
{
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
/// An editor for a value that can be parsed from a string, but with custom parsing and stringification.
pub struct CustomParseInput<T> {
    /// The current value.
    pub value: T,
    /// Callback for when the string needs to be parsed.
    pub parse: Cb<dyn Fn(&str) -> Option<T> + Sync + Send>,
    /// Callback for when the value needs to be stringified.
    pub to_string: Cb<dyn Fn(&T) -> String + Sync + Send>,
    /// Callback for when the value changes.
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}

impl<T: Debug + Clone + Sync + Send + 'static> ElementComponent for CustomParseInput<T> {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            value,
            on_change,
            parse,
            to_string,
        } = *self;

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

/// A [ParseableInput] for [f32].
pub type F32Input = ParseableInput<f32>;
/// A [ParseableInput] for [i32].
pub type I32Input = ParseableInput<i32>;
/// A [ParseableInput] for [u32].
pub type U32Input = ParseableInput<u32>;
/// A [ParseableInput] for [u64].
pub type U64Input = ParseableInput<u64>;
/// A [ParseableInput] for [usize].
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
        F32Input {
            value: self,
            on_change,
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for i32 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        I32Input {
            value: self,
            on_change,
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for u32 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        U32Input {
            value: self,
            on_change,
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for u64 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        U64Input {
            value: self,
            on_change,
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}
impl Editor for usize {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        UsizeInput {
            value: self,
            on_change,
        }
        .el()
    }

    fn view(self, _: EditorOpts) -> Element {
        Text::el(format!("{self}"))
    }
}

#[derive(Clone, Debug)]
/// A checkbox.
pub struct Checkbox {
    /// Whether or not the checkbox is checked.
    pub value: bool,
    /// Callback for when the checkbox is toggled.
    pub on_change: Cb<dyn Fn(bool) + Sync + Send>,
}
impl Checkbox {
    /// Create a new checkbox.
    pub fn new(value: bool, on_change: impl Fn(bool) + Sync + Send + 'static) -> Self {
        Self {
            value,
            on_change: cb(on_change),
        }
    }
}
impl ElementComponent for Checkbox {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Checkbox { value, on_change } = *self;
        Button::new(
            FontAwesomeIcon::el(if value { 0xf14a } else { 0xf0c8 }, false),
            move |_| on_change.0(!value),
        )
        .style(ButtonStyle::Flat)
        .el()
    }
}

impl Editor for bool {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        Checkbox {
            value: self,
            on_change,
        }
        .el()
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
/// A row with a title and an element, used as part of larger editors.

pub struct EditorRow {
    title: String,
    editor: Element,
}
impl EditorRow {
    /// Create a new editor row.
    pub fn el(title: impl Into<String>, editor: Element) -> Element {
        let title: String = title.into();
        EditorRow {
            title: title.to_case(Case::Title),
            editor,
        }
        .el()
    }
}
impl ElementComponent for EditorRow {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        let Self { title, editor } = *self;
        FlowRow(vec![
            Text::el(title).with(margin(), vec4(0., STREET, 0., 0.)),
            editor,
        ])
        .el()
    }
}

#[derive(Debug, Clone)]
/// Legacy newtype for [FlowColumn].
pub struct EditorColumn(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(EditorColumn);
impl ElementComponent for EditorColumn {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        FlowColumn(self.0).el()
    }
}

impl Editor for Vec2 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor {
            value: self.to_array(),
            on_change: cb(move |v| (on_change)(Self::from(v))),
            field_names: Some(&["X", "Y"]),
        }
        .el()
    }
}

impl Editor for Vec3 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor {
            value: self.to_array(),
            on_change: cb(move |v| (on_change)(Self::from(v))),
            field_names: Some(&["X", "Y", "Z"]),
        }
        .el()
    }
}

impl Editor for Vec4 {
    fn editor(self, on_change: ChangeCb<Self>, _: EditorOpts) -> Element {
        ArrayEditor {
            value: self.to_array(),
            on_change: cb(move |v| (on_change)(Self::from(v))),
            field_names: Some(&["X", "Y", "Z", "W"]),
        }
        .el()
    }
}

#[derive(Debug, Clone)]
/// An editor for a fixed-size array.
pub struct ArrayEditor<const C: usize, T> {
    /// The array to edit.
    pub value: [T; C],
    /// The names of the fields of the array.
    pub field_names: Option<&'static [&'static str; C]>,
    /// Callback for when the array is changed.
    pub on_change: Cb<dyn Fn([T; C]) + Sync + Send>,
}

impl<const C: usize, T: 'static + Clone + Debug + Editor + Send + Sync> ElementComponent
    for ArrayEditor<C, T>
{
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        let Self {
            value,
            on_change,
            field_names,
        } = *self;

        if let Some(field_names) = field_names {
            EditorColumn(
                value
                    .iter()
                    .enumerate()
                    .zip_eq(field_names)
                    .map(|((i, v), &name)| {
                        to_owned![value, on_change];
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
                        to_owned![value, on_change];
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
        ArrayEditor {
            value: self,
            on_change,
            field_names: None,
        }
        .el()
    }
}
