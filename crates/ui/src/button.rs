use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ambient_core::{runtime, window::WindowCtl, window_ctl};
use ambient_ecs::World;
use ambient_element::{element_component, Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_input::{on_app_focus_change, on_app_keyboard_input, on_app_mouse_input, KeyboardEvent};
use ambient_renderer::color;
use ambient_std::{cb, color::Color, Callback, Cb};
use closure::closure;
use futures::{future::BoxFuture, Future, FutureExt};
use glam::*;
use parking_lot::Mutex;
pub use winit::event::VirtualKeyCode;
use winit::{
    event::{ElementState, ModifiersState},
    window::CursorIcon,
};

use super::{FlowColumn, FlowRow, Text, UIBase, UIElement, UIExt};
use crate::{
    border_color, border_radius, border_thickness, cutout_color, font_style, layout::*, primary_color, secondary_color, Corners, FontStyle,
    Tooltip,
};

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
                world.resource(runtime()).spawn(async move {
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
                    .set(fit_horizontal(), Fit::Parent)
                    .set(height(), 2.)
                    .with_background(Color::WHITE)
                    .set(margin(), Borders::top(2.)),
            ])
            .with_background(background)
        } else {
            let content = content.set(font_style(), FontStyle::Bold);
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
                .set(
                    padding(),
                    match self {
                        Self::Card => Borders::even(3.),
                        Self::Flat => Borders::even(3.),
                        _ => Borders::rect(3., 16.),
                    },
                )
                .set(align_vertical(), Align::Center)
                .with_background(background)
                .set(
                    border_radius(),
                    match self {
                        Self::Card => Corners::even(3.),
                        Self::Flat => Corners::even(3.),
                        _ => Corners::even(26. / 2.),
                    },
                )
                .set(border_thickness(), 0.)
                .set(border_color(), Color::WHITE);
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

    let content = style
        .create_container(is_pressed, is_working, disabled, toggled, hover, hotkey, hotkey_modifier, tooltip, content)
        .with_clickarea()
        .on_mouse_enter(
            closure!(clone set_hover, |world, _| { set_hover(true); world.resource(window_ctl()).send(WindowCtl::SetCursorIcon(CursorIcon::Hand)).ok(); }),
        )
        .on_mouse_leave(move |world, _| {
            set_hover(false);
            world.resource(window_ctl()).send(WindowCtl::SetCursorIcon(CursorIcon::Default)).ok();
        })
        .listener(
            on_app_mouse_input(),
            Arc::new(closure!(clone set_is_pressed, clone on_invoked, clone set_is_working, |world, _, event| {
                if event.state == ElementState::Pressed && hover {
                    set_is_pressed(true);
                    is_pressed_immediate.store(true, Ordering::SeqCst);
                }
                if event.state == ElementState::Released {
                    let is_pressed = is_pressed_immediate.load(Ordering::SeqCst);
                    if hover && !disabled && is_pressed {
                        on_invoked.invoke(world, set_is_working.clone());
                    }
                    set_is_pressed(false);
                    is_pressed_immediate.store(false, Ordering::SeqCst);
                }
            })),
        );

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
        content
            .listener(
                on_app_focus_change(),
                Arc::new({
                    let is_pressed = is_pressed.clone();
                    move |_, _, _| {
                        is_pressed.store(false, Ordering::Relaxed);
                    }
                }),
            )
            .listener(
                on_app_keyboard_input(),
                Arc::new(move |world, _, event| {
                    if let KeyboardEvent { keycode: Some(virtual_keycode), state, modifiers, .. } = event {
                        if virtual_keycode == &hotkey {
                            if state == &ElementState::Pressed {
                                if modifiers == &hotkey_modifier {
                                    if let Some(on_is_pressed_changed) = on_is_pressed_changed.clone() {
                                        on_is_pressed_changed.0(true);
                                    }
                                    is_pressed.store(true, Ordering::Relaxed);
                                    return true;
                                }
                            } else {
                                let pressed = is_pressed.load(Ordering::Relaxed);

                                if pressed {
                                    on_invoke.0(world);
                                    if let Some(on_is_pressed_changed) = on_is_pressed_changed.clone() {
                                        on_is_pressed_changed.0(false);
                                    }
                                    is_pressed.store(false, Ordering::Relaxed);
                                    return true;
                                }
                            }
                        }
                    }
                    false
                }),
            )
    }
}
