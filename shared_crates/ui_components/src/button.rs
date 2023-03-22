use std::{
    fmt::Debug,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ambient_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use futures::{future::BoxFuture, Future, FutureExt};
use glam::*;
use parking_lot::Mutex;

use crate::{
    default_theme::{cutout_color, primary_color, secondary_color},
    dropdown::Tooltip,
    UIExt,
};
use crate::{layout::FlowColumn, layout::FlowRow, text::Text, UIBase, UIElement};
use ambient_cb::{cb, Callback, Cb};
use ambient_color::Color;
use ambient_guest_bridge::{
    components::{
        input::{event_focus_change, event_keyboard_input, event_mouse_input, keyboard_modifiers, keycode},
        layout::{
            align_vertical_center, fit_horizontal_parent, height, margin_top, min_height, padding_bottom, padding_left, padding_right,
            padding_top, space_between_items,
        },
        rect::{border_color, border_radius, border_thickness},
        rendering::color,
        text::font_style,
    },
    ecs::World,
    run_async,
};
use ambient_window_types::{CursorIcon, ModifiersState, VirtualKeyCode};

#[derive(Clone, Debug)]
pub enum ButtonCb {
    Sync(ButtonCallback),
    Async(Callback<(), BoxFuture<'static, ()>>),
}

/// The type of function invoked by a button
pub type ButtonCallback<Ret = ()> = Cb<dyn Fn(&mut World) -> Ret + Sync + Send>;

impl ButtonCb {
    pub fn invoke(&self, world: &mut World, set_is_working: Cb<dyn Fn(bool) + Sync + Send>) {
        match self {
            ButtonCb::Sync(cb) => cb.0(world),
            ButtonCb::Async(cb) => {
                set_is_working(true);
                let cb = cb.clone();
                run_async(world, async move {
                    cb.0(()).await;
                    set_is_working(false);
                });
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonStyle {
    Regular,
    Primary,
    Flat,
    /// Card buttons are meant to be used with "complex" child elements
    Card,
    Inline,
}
impl ButtonStyle {
    #[allow(clippy::too_many_arguments)]
    fn create_container(
        &self,
        is_pressed: bool,
        is_working: bool,
        disabled: bool,
        toggled: bool,
        hover: bool,
        hotkey: Option<VirtualKeyCode>,
        hotkey_modifier: ModifiersState,
        tooltip: Option<Element>,
        content: Element,
    ) -> Element {
        let background = match self {
            ButtonStyle::Regular | ButtonStyle::Card => {
                if is_pressed {
                    cutout_color().lighten(0.1)
                } else if disabled || is_working {
                    cutout_color()
                } else if toggled {
                    primary_color()
                } else if hover && *self == ButtonStyle::Card {
                    cutout_color().lighten(0.05)
                } else {
                    cutout_color()
                }
            }
            ButtonStyle::Primary => {
                if is_pressed {
                    secondary_color().lighten(0.2)
                } else if disabled || is_working {
                    secondary_color().desaturate(-1.)
                } else if hover || disabled || is_working {
                    secondary_color().lighten(0.1)
                } else {
                    secondary_color()
                }
            }
            ButtonStyle::Flat | ButtonStyle::Inline => Color::rgba(1., 1., 1., 0.0),
        };
        let content = match self {
            Self::Regular | Self::Flat | ButtonStyle::Inline => content.set(
                color(),
                if is_pressed {
                    Color::rgba(1., 1., 1., 1.)
                } else if disabled || is_working {
                    Color::rgba(0.3, 0.3, 0.3, 1.)
                } else if toggled {
                    if *self == Self::Flat || *self == Self::Inline {
                        primary_color()
                    } else {
                        Color::rgba(1., 1., 1., 1.)
                    }
                } else if hover {
                    Color::rgba(0.8, 0.8, 0.8, 1.)
                } else {
                    Color::hex("B3B3B3").unwrap()
                }
                .into(),
            ),
            Self::Primary => content.set(
                color(),
                if is_pressed {
                    Color::BLACK
                } else if disabled || is_working {
                    Color::BLACK.lighten(0.3)
                } else {
                    Color::BLACK
                }
                .into(),
            ),
            _ => content,
        };
        if *self == ButtonStyle::Inline {
            FlowColumn::el([
                content,
                UIBase
                    .el()
                    .set_default(fit_horizontal_parent())
                    .set(height(), 2.)
                    .with_background(Color::WHITE.into())
                    .set(margin_top(), 2.),
            ])
            .with_background(background.into())
        } else {
            let content = content.set(font_style(), "Bold".to_string());
            let tooltip = if let Some(hotkey) = hotkey {
                let modifier = if hotkey_modifier != ModifiersState::empty() { format!("{hotkey_modifier:?} + ") } else { String::new() };
                let hotkey = Text::el(format!("[{modifier}{hotkey:?}]"));
                if let Some(tooltip) = tooltip {
                    Some(FlowColumn::el([tooltip, hotkey]).set(space_between_items(), 10.))
                } else {
                    Some(hotkey)
                }
            } else {
                tooltip
            };
            let mut el = FlowRow(vec![content])
                .el()
                .set(padding_top(), 3.)
                .set(padding_bottom(), 3.)
                .set(padding_left(), if matches!(self, Self::Card) || matches!(self, Self::Flat) { 3. } else { 16. })
                .set(padding_right(), if matches!(self, Self::Card) || matches!(self, Self::Flat) { 3. } else { 16. })
                .set_default(align_vertical_center())
                .with_background(background.into())
                .set(
                    border_radius(),
                    match self {
                        Self::Card => Vec4::ONE * 3.,
                        Self::Flat => Vec4::ONE * 3.,
                        _ => Vec4::ONE * 26. / 2.,
                    },
                )
                .set(border_thickness(), 0.)
                .set(border_color(), Color::WHITE.into());
            if *self != Self::Flat {
                el = el.set(min_height(), 26.);
            }
            if let Some(tooltip) = tooltip {
                Tooltip { inner: el, tooltip }.el()
            } else {
                el
            }
        }
    }
}

#[element_component]
pub fn Button(
    hooks: &mut Hooks,
    content: Element,
    disabled: bool,
    toggled: bool,
    style: ButtonStyle,
    hotkey: Option<VirtualKeyCode>,
    hotkey_modifier: ModifiersState,
    tooltip: Option<Element>,
    on_invoked: ButtonCb,
    on_is_pressed_changed: Option<Cb<dyn Fn(&mut World, bool) + Sync + Send>>,
) -> Element {
    let (is_pressed, set_is_pressed) = hooks.use_state(false);
    let (hover, set_hover) = hooks.use_state(false);
    let (is_working, set_is_working) = hooks.use_state(false);
    let (is_pressed_immediate, _) = hooks.use_state_with(|_| Arc::new(AtomicBool::new(false)));

    hooks.use_effect(is_pressed, move |world, _| {
        if let Some(on_is_pressed_changed) = on_is_pressed_changed {
            on_is_pressed_changed(world, is_pressed);
        }
        Box::new(|_| {})
    });
    hooks.use_event(ambient_event_types::WINDOW_MOUSE_INPUT, {
        let set_is_pressed = set_is_pressed.clone();
        let on_invoked = on_invoked.clone();
        let set_is_working = set_is_working.clone();
        move |world, event| {
            if let Some(pressed) = event.get(event_mouse_input()) {
                if pressed && hover {
                    set_is_pressed(true);
                    is_pressed_immediate.store(true, Ordering::SeqCst);
                }
                if !pressed {
                    let is_pressed = is_pressed_immediate.load(Ordering::SeqCst);
                    if hover && !disabled && is_pressed {
                        on_invoked.invoke(world, set_is_working.clone());
                    }
                    set_is_pressed(false);
                    is_pressed_immediate.store(false, Ordering::SeqCst);
                }
            }
        }
    });

    let content = style
        .create_container(is_pressed, is_working, disabled, toggled, hover, hotkey, hotkey_modifier, tooltip, content)
        .with_clickarea()
        .on_mouse_enter({
            let set_hover = set_hover.clone();
            move |world, _| {
                set_hover(true);
                ambient_guest_bridge::window::set_cursor(world, CursorIcon::Hand);
            }
        })
        .on_mouse_leave(move |world, _| {
            set_hover(false);
            ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
        })
        .el();

    if disabled {
        content
    } else if let Some(hotkey) = hotkey {
        Hotkey {
            hotkey,
            hotkey_modifier,
            content,
            on_is_pressed_changed: Some(set_is_pressed),
            on_invoke: cb(move |world| {
                on_invoked.invoke(world, set_is_working.clone());
            }),
        }
        .el()
    } else {
        content
    }
}
impl Button {
    pub fn new<T: Into<UIElement>>(content: T, on_invoked: impl Fn(&mut World) + Sync + Send + 'static) -> Self {
        let content: UIElement = content.into();
        Self::new_inner(content, cb(on_invoked))
    }
    pub fn new_inner<T: Into<UIElement>>(content: T, on_invoked: Cb<dyn Fn(&mut World) + Sync + Send + 'static>) -> Self {
        let content: UIElement = content.into();
        Self {
            content: content.0,
            disabled: false,
            toggled: false,
            style: ButtonStyle::Regular,
            hotkey: None,
            hotkey_modifier: ModifiersState::empty(),
            tooltip: None,
            on_invoked: ButtonCb::Sync(on_invoked),
            on_is_pressed_changed: None,
        }
    }
    pub fn new_once<T: Into<UIElement>>(content: T, on_invoked: impl FnOnce(&mut World) + Sync + Send + 'static) -> Self {
        let on_invoked = Arc::new(Mutex::new(Some(on_invoked)));
        Self::new(content, move |world| {
            let on_invoked = on_invoked.clone();
            let on_invoked = {
                let mut on_invoked = on_invoked.lock();
                std::mem::replace(&mut *on_invoked, None)
            };
            on_invoked.expect("'Once' button called more than once")(world);
        })
    }
    pub fn new_async<F: Future<Output = ()> + Send + 'static, T: Into<UIElement>>(
        content: T,
        on_invoked: impl Fn() -> F + Sync + Send + 'static,
    ) -> Self {
        let content: UIElement = content.into();
        Self {
            content: content.0,
            disabled: false,
            toggled: false,
            style: ButtonStyle::Regular,
            hotkey: None,
            hotkey_modifier: ModifiersState::empty(),
            tooltip: None,
            on_invoked: ButtonCb::Async(cb(move |_w| on_invoked().boxed())),
            on_is_pressed_changed: None,
        }
    }
    pub fn new_async_once<F: Future + Send + 'static, T: Into<UIElement>>(
        content: T,
        on_invoked: impl FnOnce() -> F + Sync + Send + 'static,
    ) -> Self {
        let on_invoked = Arc::new(Mutex::new(Some(on_invoked)));
        Self::new_async(content, move || {
            let on_invoked = on_invoked.clone();
            async move {
                let on_invoked = {
                    let mut on_invoked = on_invoked.lock();
                    std::mem::replace(&mut *on_invoked, None)
                };
                on_invoked.expect("'Async once' button called more than once")().await;
            }
            .boxed()
        })
    }
    pub fn new_value<T: Into<UIElement>, V: PartialEq + Copy + Send + Sync + 'static>(
        content: T,
        value: V,
        set_value: Cb<dyn Fn(V) + Sync + Send>,
        desired_value: V,
    ) -> Button {
        Button::new(content, move |_| set_value(desired_value)).toggled(value == desired_value)
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
    pub fn hotkey(mut self, hotkey: VirtualKeyCode) -> Self {
        self.hotkey = Some(hotkey);
        self
    }
    pub fn hotkey_modifier(mut self, hotkey_modifier: ModifiersState) -> Self {
        self.hotkey_modifier = hotkey_modifier;
        self
    }
    pub fn tooltip(mut self, tooltip: impl Into<UIElement>) -> Self {
        let tooltip: UIElement = tooltip.into();
        self.tooltip = Some(tooltip.0);
        self
    }
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }
    pub fn on_is_pressed_changed(mut self, handle: impl Fn(&mut World, bool) + Sync + Send + 'static) -> Self {
        self.on_is_pressed_changed = Some(cb(handle));
        self
    }
}

#[derive(Clone, Debug)]
pub struct Hotkey {
    pub hotkey: VirtualKeyCode,
    pub hotkey_modifier: ModifiersState,
    pub on_is_pressed_changed: Option<Cb<dyn Fn(bool) + Sync + Send>>,
    pub on_invoke: Cb<dyn Fn(&mut World) + Sync + Send>,
    pub content: Element,
}
impl Hotkey {
    pub fn new(hotkey: VirtualKeyCode, on_invoke: impl Fn(&mut World) + Sync + Send + 'static, content: Element) -> Self {
        Self { hotkey, hotkey_modifier: ModifiersState::empty(), on_invoke: cb(on_invoke), content, on_is_pressed_changed: None }
    }
    pub fn hotkey_modifier(mut self, hotkey_modifier: ModifiersState) -> Self {
        self.hotkey_modifier = hotkey_modifier;
        self
    }
}
impl ElementComponent for Hotkey {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { on_is_pressed_changed, content, hotkey, hotkey_modifier, on_invoke } = *self;
        let (is_pressed, _) = hooks.use_state_with(|_| Arc::new(AtomicBool::new(false)));
        hooks.use_event(ambient_event_types::WINDOW_KEYBOARD_INPUT, {
            let is_pressed = is_pressed.clone();
            move |world, event| {
                if let Some(pressed) = event.get(event_keyboard_input()) {
                    let modifiers = ModifiersState::from_bits(event.get(keyboard_modifiers()).unwrap()).unwrap();

                    // FIXME: get_ref returns `&T` on native, but `T` on guest
                    if let Some(virtual_keycode) = event.get_ref(keycode()).and_then(|x| VirtualKeyCode::from_str(&x).ok()) {
                        let shortcut_pressed = modifiers == hotkey_modifier && virtual_keycode == hotkey;
                        if shortcut_pressed {
                            on_invoke.0(world);
                            if pressed {
                                if let Some(on_is_pressed_changed) = on_is_pressed_changed.clone() {
                                    on_is_pressed_changed.0(true);
                                }
                                is_pressed.store(true, Ordering::Relaxed);
                            } else {
                                if let Some(on_is_pressed_changed) = on_is_pressed_changed.clone() {
                                    on_is_pressed_changed.0(false);
                                }
                                is_pressed.store(false, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        });
        hooks.use_event(ambient_event_types::WINDOW_FOCUSED, {
            move |_world, event| {
                if let Some(_event) = event.get(event_focus_change()) {
                    is_pressed.store(false, Ordering::Relaxed);
                }
            }
        });
        content
    }
}
