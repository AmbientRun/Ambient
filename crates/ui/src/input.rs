use std::{
    self, f32::consts::E, fmt::Debug, hash::Hash, str::FromStr, sync::Arc, time::{Duration, SystemTime}
};

use elements_core::{mouse_position, on_event, transform::translation, window, window_scale_factor};
use elements_ecs::{ComponentValue, EntityId, World};
use elements_element::{define_el_function_for_vec_element_newtype, Element, ElementComponent, ElementComponentExt, Hooks};
use elements_input::MouseButton;
use elements_std::{
    events::EventDispatcher, math::{interpolate, interpolate_clamped, Saturate}, Cb
};
use glam::*;
use winit::{
    event::{ElementState, Event, WindowEvent}, window::CursorIcon
};

use super::{Editor, EditorOpts, FlowColumn, FlowRow, Focus, Text, UIBase, UIExt};
use crate::{
    background_color, border_radius, layout::*, primary_color, text_input::TextInput, Button, ButtonStyle, Corners, FontAwesomeIcon, Rectangle, STREET
};

#[derive(Debug, Clone)]
pub struct ParseableInput<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> {
    pub value: T,
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}
impl<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> ParseableInput<T> {
    pub fn new(value: T, on_change: impl Fn(T) + Sync + Send + 'static) -> Self {
        Self { value, on_change: Cb::new(on_change) }
    }
}
impl<T: FromStr + Debug + std::fmt::Display + Clone + Sync + Send + 'static> ElementComponent for ParseableInput<T> {
    fn render(self: Box<Self>, _: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        let (self_id, set_self_id) = hooks.use_state(EntityId::null());
        let (focus, _) = hooks.consume_context::<Focus>().expect("No FocusRoot found");
        let focused = focus == Focus(Some(self_id));
        let (text, set_text) = hooks.use_state(None);
        if focused && text.is_none() {
            set_text(Some(value.to_string()));
        } else if !focused && text.is_some() {
            set_text(None);
        }
        TextInput::new(
            text.unwrap_or_else(|| value.to_string()),
            Cb(Arc::new(move |text| {
                if let Ok(value) = text.parse::<T>() {
                    on_change.0(value);
                }
                set_text(Some(text));
            })),
        )
        .el()
        .on_spawned(move |_, id| set_self_id(id))
    }
}

#[derive(Debug, Clone)]
pub struct CustomParseInput<T> {
    pub value: T,
    pub parse: Cb<dyn Fn(&str) -> Option<T> + Sync + Send>,
    pub to_string: Cb<dyn Fn(&T) -> String + Sync + Send>,
    pub on_change: Cb<dyn Fn(T) + Sync + Send>,
}

impl<T: Debug + ComponentValue> ElementComponent for CustomParseInput<T> {
    fn render(self: Box<Self>, _: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change, parse, to_string } = *self;

        let (self_id, set_self_id) = hooks.use_state(EntityId::null());
        let (focus, _) = hooks.consume_context::<Focus>().expect("No FocusRoot found");
        let focused = focus == Focus(Some(self_id));
        let (text, set_text) = hooks.use_state(None);
        if focused && text.is_none() {
            set_text(Some(to_string(&value)));
        } else if !focused && text.is_some() {
            set_text(None);
        }
        TextInput::new(
            text.unwrap_or_else(|| to_string(&value)),
            Cb(Arc::new(move |text| {
                if let Some(value) = parse(&text) {
                    on_change.0(value);
                }
                set_text(Some(text));
            })),
        )
        .el()
        .on_spawned(move |_, id| set_self_id(id))
    }
}

pub type F32Input = ParseableInput<f32>;
pub type I32Input = ParseableInput<i32>;
pub type U32Input = ParseableInput<u32>;
pub type U64Input = ParseableInput<u64>;
pub type UsizeInput = ParseableInput<usize>;

impl Editor for Duration {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            CustomParseInput {
                value: self,
                on_change,
                parse: Cb::new(|v| v.parse::<f32>().ok().map(Duration::from_secs_f32)),
                to_string: Cb::new(|v| format!("{:.3}", v.as_secs_f32())),
            }
            .el()
        } else {
            Text::el(format!("{}", self.as_secs_f32()))
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

impl Editor for f32 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            F32Input { value: self, on_change }.el()
        } else {
            Text::el(format!("{self}"))
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}
impl Editor for i32 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            I32Input { value: self, on_change }.el()
        } else {
            Text::el(format!("{self}"))
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

impl Editor for u32 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            U32Input { value: self, on_change }.el()
        } else {
            Text::el(format!("{self}"))
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}
impl Editor for u64 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            U64Input { value: self, on_change }.el()
        } else {
            Text::el(format!("{self}"))
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}
impl Editor for usize {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            UsizeInput { value: self, on_change }.el()
        } else {
            Text::el(format!("{self}"))
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

#[derive(Clone, Debug)]
pub struct Checkbox {
    pub value: bool,
    pub on_change: Cb<dyn Fn(bool) + Sync + Send>,
}
impl Checkbox {
    pub fn new(value: bool, on_change: impl Fn(bool) + Sync + Send + 'static) -> Self {
        Self { value, on_change: Cb(Arc::new(on_change)) }
    }
}
impl ElementComponent for Checkbox {
    fn render(self: Box<Self>, _world: &mut World, _: &mut Hooks) -> Element {
        let Checkbox { value, on_change } = *self;
        Button::new(FontAwesomeIcon::el(if value { 0xf14a } else { 0xf0c8 }, false), move |_| on_change.0(!value))
            .style(ButtonStyle::Flat)
            .el()
    }
}

impl Editor for bool {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            Checkbox { value: self, on_change }.el()
        } else if self {
            FontAwesomeIcon::el(0xf14a, false)
        } else {
            FontAwesomeIcon::el(0xf0c8, false)
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

#[derive(Clone, Debug)]
pub struct Slider {
    pub value: f32,
    pub on_change: Option<Cb<dyn Fn(f32) + Sync + Send>>,
    pub min: f32,
    pub max: f32,
    pub width: f32,
    pub logarithmic: bool,
    pub round: Option<u32>,
    pub suffix: Option<&'static str>,
}

impl ElementComponent for Slider {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Slider { value, min, max, width: slider_width, logarithmic, round, suffix, .. } = *self;
        const THUMB_WIDTH: f32 = 12.;
        const SLIDER_HEIGHT: f32 = 12.;

        fn cleanup_value(value: f32, min: f32, max: f32, round: Option<u32>) -> f32 {
            let mut processed = value.clamp(min, max);

            if let Some(decimal_precision) = round {
                let exponent = 10.0f32.powi(decimal_precision as i32);
                processed = (processed * exponent).round() / exponent;
            }

            processed
        }

        let value = cleanup_value(value, min, max, round);
        hooks.use_spawn({
            let on_change = self.on_change.clone();
            let old_value = self.value;
            move |_| {
                if old_value != value {
                    if let Some(on_change) = on_change {
                        on_change(value);
                    }
                }

                Box::new(|_| {})
            }
        });

        // Sets the value with some sanitization
        let on_change_raw = self
            .on_change
            .map(|f| Cb(Arc::new(move |value: f32| f(cleanup_value(value, min, max, round))) as Arc<dyn Fn(f32) + Sync + Send>));
        // Sets the value after converting from [0-1] to the value range
        let on_change_factor = on_change_raw.clone().map(|f| {
            Cb(Arc::new(move |p: f32| f(if logarithmic { p.powf(E) } else { p } * (max - min) + min)) as Arc<dyn Fn(f32) + Sync + Send>)
        });

        let rectangle = Rectangle
            .el()
            .set(width(), slider_width)
            .set(height(), 2.)
            .set(translation(), vec3(0., (SLIDER_HEIGHT - 2.) / 2., 0.))
            .set(background_color(), primary_color());

        let thumb = {
            let p = interpolate(value, min, max, 0., 1.);
            let block_left_offset = if logarithmic { p.powf(1. / E) } else { p } * (slider_width - THUMB_WIDTH);
            let block_left_offset = if block_left_offset.is_nan() || block_left_offset.is_infinite() { 0. } else { block_left_offset };
            // f(x) = p ^ e
            // f'(f(x)) = x
            // f'(y) = y ^ (1/e)
            // (p ^ e) ^ (1/e) = p ^ (e / e) = p ^ 1 = p

            let thumb = UIBase
                .el()
                .set(width(), THUMB_WIDTH)
                .set(height(), SLIDER_HEIGHT)
                .with_background(primary_color())
                .set(border_radius(), Corners::even(THUMB_WIDTH / 2.))
                .set(translation(), vec3(block_left_offset, 0., -0.01))
                .on_mouse_enter(|world, _| world.resource(window()).set_cursor_icon(CursorIcon::Hand))
                .on_mouse_leave(|world, _| world.resource(window()).set_cursor_icon(CursorIcon::Default));
            if let Some(on_change_factor) = on_change_factor.clone() {
                thumb.on_mouse_down(move |world, id, _| {
                    let on_change_factor = on_change_factor.clone();
                    let scale_factor = *world.resource(window_scale_factor());
                    let start_pos = *world.resource(mouse_position()) / scale_factor as f32;
                    let screen_min = start_pos.x - block_left_offset;
                    let screen_max = screen_min + slider_width - THUMB_WIDTH;
                    world
                        .add_component(
                            id,
                            on_event(),
                            EventDispatcher::new_with(Arc::new(move |world, id, event| match event {
                                Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                                    let x = position.x as f32 / scale_factor as f32;
                                    on_change_factor(interpolate_clamped(x, screen_min, screen_max, 0., 1.));
                                }
                                Event::WindowEvent { event: WindowEvent::MouseInput { state: ElementState::Released, .. }, .. } => {
                                    world.remove_component(id, on_event()).unwrap();
                                }
                                _ => {}
                            })),
                        )
                        .unwrap();
                })
            } else {
                thumb
            }
        };

        FlowRow::el([
            UIBase.el().set(width(), slider_width).set(height(), SLIDER_HEIGHT).children(vec![rectangle, thumb]).on_mouse_up(
                move |world, id, button| {
                    if let Some(on_change_factor) = on_change_factor.clone() {
                        if button != MouseButton::Left {
                            return;
                        }
                        let scale_factor = *world.resource(window_scale_factor());
                        let mouse_pos = *world.resource(mouse_position()) / scale_factor as f32;

                        let screen_to_local = world.get(id, elements_core::transform::mesh_to_world()).unwrap_or_default().inverse();
                        let mouse_pos_relative = screen_to_local * Vec4::from((mouse_pos, 0.0, 1.0));

                        on_change_factor((mouse_pos_relative.x / slider_width).saturate());
                    }
                },
            ),
            FlowRow::el([f32::editor(value, on_change_raw, EditorOpts::default()), suffix.map(|s| Text::el(s)).unwrap_or_default()]),
        ])
        .set(space_between_items(), STREET)
    }
}

#[derive(Clone, Debug)]
pub struct IntegerSlider {
    pub value: i32,
    pub on_change: Option<Cb<dyn Fn(i32) + Sync + Send>>,
    pub min: i32,
    pub max: i32,
    pub width: f32,
    pub logarithmic: bool,
    pub suffix: Option<&'static str>,
}
impl ElementComponent for IntegerSlider {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let Self { value, on_change, min, max, width, logarithmic, suffix } = *self;
        Slider {
            value: value as f32,
            on_change: on_change
                .map(|on_change| Cb(Arc::new(move |value: f32| on_change.0(value as i32)) as Arc<dyn Fn(f32) + Sync + Send>)),
            min: min as f32,
            max: max as f32,
            width,
            logarithmic,
            round: None,
            suffix,
        }
        .el()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct EditableDuration {
    pub(crate) dur: Duration,
    valid: bool,
    input: String,
}

impl EditableDuration {
    pub fn new(dur: Duration, valid: bool, input: String) -> Self {
        Self { dur, valid, input }
    }

    pub fn dur(&self) -> Duration {
        self.dur
    }
}

impl From<Duration> for EditableDuration {
    fn from(v: Duration) -> Self {
        Self { dur: v, input: format!("{}s", v.as_secs()), valid: true }
    }
}

impl From<EditableDuration> for Duration {
    fn from(v: EditableDuration) -> Self {
        v.dur
    }
}

impl From<&EditableDuration> for Duration {
    fn from(v: &EditableDuration) -> Self {
        v.dur
    }
}
use convert_case::{Case, Casing};
use elements_renderer::color;
use elements_std::time::parse_duration;

impl From<String> for EditableDuration {
    fn from(s: String) -> Self {
        let dur = parse_duration(&s);
        let valid = dur.is_ok();
        Self { dur: dur.unwrap_or_default(), valid, input: s }
    }
}

#[derive(Debug, Clone)]
pub struct DurationEditor {
    pub value: EditableDuration,
    pub on_change: Cb<dyn Fn(Duration) + Sync + Send>,
}

impl DurationEditor {
    pub fn new(value: EditableDuration, on_change: Cb<dyn Fn(Duration) + Sync + Send>) -> Self {
        Self { value, on_change }
    }
}

impl ElementComponent for DurationEditor {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let Self { value: EditableDuration { input, dur, valid }, on_change } = *self;
        let input = TextInput::new(input, Cb(Arc::new(move |upd: String| on_change(EditableDuration::from(upd).dur)))).el();
        let value = Text::el(format!("{dur:#?}"));

        if valid {
            FlowRow(vec![input, value]).el().set(space_between_items(), 10.0)
        } else {
            FlowRow(vec![input, Text::el("invalid duration").set(color(), vec4(1.0, 0.0, 0.0, 1.0))]).el().set(space_between_items(), 10.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemTimeEditor {
    pub value: SystemTime,
    pub on_change: Option<Cb<dyn Fn(SystemTime) + Sync + Send>>,
}

impl ElementComponent for SystemTimeEditor {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        Text::el(format!("{:?}", self.value))
    }
}
impl Editor for SystemTime {
    type Output = Self;

    fn editor(self, _: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        SystemTimeEditor { value: self, on_change: None }.el()
    }

    fn value(&self) -> Self::Output {
        *self
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
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        let Self { title, editor } = *self;
        FlowRow(vec![Text::el(title).set(margin(), Borders::right(STREET)), editor]).el()
    }
}
#[derive(Debug, Clone)]
pub struct EditorColumn(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(EditorColumn);
impl ElementComponent for EditorColumn {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        FlowColumn(self.0).el()
    }
}

#[derive(Debug, Clone)]
pub struct Vec2Editor {
    pub value: Vec2,
    pub on_change: Cb<dyn Fn(Vec2) + Sync + Send>,
}
impl ElementComponent for Vec2Editor {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        EditorColumn(vec![
            EditorRow::el(
                "X",
                F32Input::new(value.x, {
                    let on_change = on_change.clone();
                    move |x| on_change(vec2(x, value.y))
                })
                .el(),
            ),
            EditorRow::el(
                "Y",
                F32Input::new(value.y, {
                    let on_change = on_change.clone();
                    move |y| on_change(vec2(value.x, y))
                })
                .el(),
            ),
        ])
        .el()
    }
}

impl Editor for Vec2 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            Vec2Editor { value: self, on_change }.el()
        } else {
            Text::el(self.to_string())
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

#[derive(Debug, Clone)]
pub struct Vec3Editor {
    pub value: Vec3,
    pub on_change: Cb<dyn Fn(Vec3) + Sync + Send>,
}

impl ElementComponent for Vec3Editor {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        EditorColumn(vec![
            EditorRow::el(
                "X",
                F32Input::new(value.x, {
                    let on_change = on_change.clone();
                    move |x| on_change(vec3(x, value.y, value.z))
                })
                .el(),
            ),
            EditorRow::el(
                "Y",
                F32Input::new(value.y, {
                    let on_change = on_change.clone();
                    move |y| on_change(vec3(value.x, y, value.z))
                })
                .el(),
            ),
            EditorRow::el(
                "Z",
                F32Input::new(value.z, {
                    let on_change = on_change.clone();
                    move |z| on_change(vec3(value.x, value.y, z))
                })
                .el(),
            ),
        ])
        .el()
    }
}

impl Editor for Vec3 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            Vec3Editor { value: self, on_change }.el()
        } else {
            Text::el(self.to_string())
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}

#[derive(Debug, Clone)]
pub struct Vec4Editor {
    pub value: Vec4,
    pub on_change: Cb<dyn Fn(Vec4) + Sync + Send>,
}

impl ElementComponent for Vec4Editor {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        EditorColumn(vec![
            EditorRow::el(
                "X",
                F32Input::new(value.x, {
                    let on_change = on_change.clone();
                    move |x| on_change(vec4(x, value.y, value.z, value.w))
                })
                .el(),
            ),
            EditorRow::el(
                "Y",
                F32Input::new(value.y, {
                    let on_change = on_change.clone();
                    move |y| on_change(vec4(value.x, y, value.z, value.w))
                })
                .el(),
            ),
            EditorRow::el(
                "Z",
                F32Input::new(value.z, {
                    let on_change = on_change.clone();
                    move |z| on_change(vec4(value.x, value.y, z, value.w))
                })
                .el(),
            ),
            EditorRow::el(
                "W",
                F32Input::new(value.w, {
                    let on_change = on_change.clone();
                    move |w| on_change(vec4(value.x, value.y, value.z, w))
                })
                .el(),
            ),
        ])
        .el()
    }
}

impl Editor for Vec4 {
    type Output = Self;

    fn editor(self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            Vec4Editor { value: self, on_change }.el()
        } else {
            Text::el(self.to_string())
        }
    }

    fn value(&self) -> Self::Output {
        *self
    }
}
