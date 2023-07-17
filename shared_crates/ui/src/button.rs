//! Implements a button UI element, as well as variants thereof.

use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ambient_cb::{cb, Callback, Cb};
use ambient_color::Color;
use ambient_element::{
    element_component, to_owned, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::{
    components::{
        layout::{
            align_vertical_center, fit_horizontal_parent, height, margin, min_height, padding,
            space_between_items,
        },
        rect::{border_color, border_radius, border_thickness},
        rendering::color,
        text::font_style,
    },
    ecs::World,
    messages, run_async,
};
use ambient_shared_types::{CursorIcon, ModifiersState, VirtualKeyCode};
use futures::{future::BoxFuture, Future, FutureExt};
use glam::*;
use parking_lot::Mutex;

use crate::{
    default_theme::{cutout_color, primary_color, secondary_color},
    dropdown::Tooltip,
    layout::{FlowColumn, FlowRow},
    text::Text,
    HooksExt, UIBase, UIElement, UIExt,
};

#[derive(Clone, Debug)]
/// The callback invoked when a button is clicked.
pub enum ButtonCb {
    /// A synchronous callback.
    Sync(ButtonCallback),
    /// An asynchronous callback.
    Async(Callback<(), BoxFuture<'static, ()>>),
}

/// The type of function invoked by a button.
pub type ButtonCallback<Ret = ()> = Cb<dyn Fn(&mut World) -> Ret + Sync + Send>;

impl ButtonCb {
    /// Invokes this callback.
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
/// The style of a button.
pub enum ButtonStyle {
    /// A regular button: uses variants of the cutout color for all states except hover, where it uses the secondary color.
    Regular,
    /// A primary button: uses variants of the secondary color.
    Primary,
    /// A flat button: has no background, has limited padding.
    Flat,
    /// A card button: meant to be used with "complex" child elements.
    Card,
    /// An inline button: has no additional styling aside from an underline, similar to a link on a webpage.
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
            Self::Regular | Self::Flat | ButtonStyle::Inline => content.with(
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
            Self::Primary => content.with(
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
                    .with_default(fit_horizontal_parent())
                    .with(height(), 2.)
                    .with_background(Color::WHITE.into())
                    .with(margin(), vec4(2., 0., 0., 0.)),
            ])
            .with_background(background.into())
        } else {
            let content = content.with(font_style(), "Bold".to_string());
            let tooltip = if let Some(hotkey) = hotkey {
                let modifier = if hotkey_modifier != ModifiersState::empty() {
                    format!("{hotkey_modifier:?} + ")
                } else {
                    String::new()
                };
                let hotkey = Text::el(format!("[{modifier}{hotkey:?}]"));
                if let Some(tooltip) = tooltip {
                    Some(FlowColumn::el([tooltip, hotkey]).with(space_between_items(), 10.))
                } else {
                    Some(hotkey)
                }
            } else {
                tooltip
            };
            let mut el = FlowRow(vec![content])
                .el()
                .with(
                    padding(),
                    vec4(
                        3.,
                        if matches!(self, Self::Card) || matches!(self, Self::Flat) {
                            3.
                        } else {
                            16.
                        },
                        3.,
                        if matches!(self, Self::Card) || matches!(self, Self::Flat) {
                            3.
                        } else {
                            16.
                        },
                    ),
                )
                .with_default(align_vertical_center())
                .with_background(background.into())
                .with(
                    border_radius(),
                    match self {
                        Self::Card => Vec4::ONE * 3.,
                        Self::Flat => Vec4::ONE * 3.,
                        _ => Vec4::ONE * 26. / 2.,
                    },
                )
                .with(border_thickness(), 0.)
                .with(border_color(), Color::WHITE.into());
            if *self != Self::Flat {
                el = el.with(min_height(), 26.);
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
/// A button UI element.
pub fn Button(
    hooks: &mut Hooks,
    /// The content of the button; typically text, as provided by one of the constructor functions.
    content: Element,
    /// Whether or not the button is disabled.
    disabled: bool,
    /// Whether or not the button can be toggled.
    toggled: bool,
    /// The style of the button.
    style: ButtonStyle,
    /// The hotkey for the button.
    hotkey: Option<VirtualKeyCode>,
    /// The hotkey input modifiers for the button.
    hotkey_modifier: ModifiersState,
    /// The tooltip for the button.
    tooltip: Option<Element>,
    /// The callback to invoke when the button is pressed.
    on_invoked: ButtonCb,
    /// The callback to invoke when the current pressed state changes.
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
        |_| {}
    });
    hooks.use_runtime_message::<messages::WindowMouseInput>({
        to_owned![set_is_pressed, on_invoked, set_is_working];
        move |world, event| {
            let pressed = event.pressed;
            if pressed && hover {
                set_is_pressed(true);
                is_pressed_immediate.store(true, Ordering::SeqCst);
            }
            if !pressed {
                let is_pressed = is_pressed_immediate.load(Ordering::SeqCst);
                if hover && !disabled && is_pressed {
                    on_invoked.invoke(world, set_is_working.clone());
                }
                if is_pressed {
                    set_is_pressed(false);
                    is_pressed_immediate.store(false, Ordering::SeqCst);
                }
            }
        }
    });

    let content = style
        .create_container(
            is_pressed,
            is_working,
            disabled,
            toggled,
            hover,
            hotkey,
            hotkey_modifier,
            tooltip,
            content,
        )
        .with_clickarea()
        .on_mouse_enter({
            to_owned![set_hover];
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
    /// Create a new [Button] with the given content and callback.
    pub fn new<T: Into<UIElement>>(
        content: T,
        on_invoked: impl Fn(&mut World) + Sync + Send + 'static,
    ) -> Self {
        let content: UIElement = content.into();
        Self::new_inner(content, cb(on_invoked))
    }
    /// Create a new [Button] with the given content and callback as a [Cb].
    pub fn new_inner<T: Into<UIElement>>(
        content: T,
        on_invoked: Cb<dyn Fn(&mut World) + Sync + Send + 'static>,
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
            on_invoked: ButtonCb::Sync(on_invoked),
            on_is_pressed_changed: None,
        }
    }
    /// Create a new one-shot [Button] with the given content and a callback that is only invoked once.
    pub fn new_once<T: Into<UIElement>>(
        content: T,
        on_invoked: impl FnOnce(&mut World) + Sync + Send + 'static,
    ) -> Self {
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
    /// Create a new [Button] with the given content and a callback that returns a [Future] (i.e. is `async`).
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
    /// Create a new one-shot [Button] with the given content and a callback that returns a [Future] (i.e. is `async`) and is only invoked once.
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
    /// Create a new [Button] with the given content that sets the given value to the given desired value when clicked.
    pub fn new_value<T: Into<UIElement>, V: PartialEq + Copy + Send + Sync + 'static>(
        content: T,
        value: V,
        set_value: Cb<dyn Fn(V) + Sync + Send>,
        desired_value: V,
    ) -> Button {
        Button::new(content, move |_| set_value(desired_value)).toggled(value == desired_value)
    }
    /// Set whether or not the button is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Set the style of the button.
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }
    /// Set the hotkey of the button.
    pub fn hotkey(mut self, hotkey: VirtualKeyCode) -> Self {
        self.hotkey = Some(hotkey);
        self
    }
    /// Set the modifiers for the hotkey of the button.
    pub fn hotkey_modifier(mut self, hotkey_modifier: ModifiersState) -> Self {
        self.hotkey_modifier = hotkey_modifier;
        self
    }
    /// Set the tooltip of the button.
    pub fn tooltip(mut self, tooltip: impl Into<UIElement>) -> Self {
        let tooltip: UIElement = tooltip.into();
        self.tooltip = Some(tooltip.0);
        self
    }
    /// Set whether or not the button can be toggled.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }
    /// Set the callback that is invoked when the current pressed state of the button changes.
    pub fn on_is_pressed_changed(
        mut self,
        handle: impl Fn(&mut World, bool) + Sync + Send + 'static,
    ) -> Self {
        self.on_is_pressed_changed = Some(cb(handle));
        self
    }
}

#[derive(Clone, Debug)]
/// An element that will invoke a callback when a hotkey is pressed.
pub struct Hotkey {
    /// The hotkey that will invoke the callback.
    pub hotkey: VirtualKeyCode,
    /// The keyboard modifiers for the hotkey.
    pub hotkey_modifier: ModifiersState,
    /// The callback that is invoked when the current state of the hotkey changes.
    pub on_is_pressed_changed: Option<Cb<dyn Fn(bool) + Sync + Send>>,
    /// The callback that is invoked when the hotkey is pressed.
    pub on_invoke: Cb<dyn Fn(&mut World) + Sync + Send>,
    /// What to render for this element.
    pub content: Element,
}
impl Hotkey {
    /// Create a new [Hotkey] with the given content and callback.
    pub fn new(
        hotkey: VirtualKeyCode,
        on_invoke: impl Fn(&mut World) + Sync + Send + 'static,
        content: Element,
    ) -> Self {
        Self {
            hotkey,
            hotkey_modifier: ModifiersState::empty(),
            on_invoke: cb(on_invoke),
            content,
            on_is_pressed_changed: None,
        }
    }
    /// Set the keyboard modifiers for the hotkey.
    pub fn hotkey_modifier(mut self, hotkey_modifier: ModifiersState) -> Self {
        self.hotkey_modifier = hotkey_modifier;
        self
    }
}
impl ElementComponent for Hotkey {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self {
            on_is_pressed_changed,
            content,
            hotkey,
            hotkey_modifier,
            on_invoke,
        } = *self;
        let (is_pressed, _) = hooks.use_state_with(|_| Arc::new(AtomicBool::new(false)));
        hooks.use_keyboard_input({
            let is_pressed = is_pressed.clone();
            move |world, keycode, modifiers, pressed| {
                if let Some(virtual_keycode) = keycode {
                    if virtual_keycode != hotkey {
                        return;
                    }
                    if pressed {
                        if modifiers == hotkey_modifier {
                            if let Some(on_is_pressed_changed) = on_is_pressed_changed.clone() {
                                on_is_pressed_changed.0(true);
                            }
                            is_pressed.store(true, Ordering::Relaxed);
                        }
                    } else {
                        let pressed = is_pressed.load(Ordering::Relaxed);

                        if pressed {
                            on_invoke.0(world);
                            if let Some(on_is_pressed_changed) = on_is_pressed_changed.clone() {
                                on_is_pressed_changed.0(false);
                            }
                            is_pressed.store(false, Ordering::Relaxed);
                        }
                    }
                }
            }
        });
        hooks.use_runtime_message::<messages::WindowFocusChange>({
            move |_world, _event| {
                is_pressed.store(false, Ordering::Relaxed);
            }
        });
        content
    }
}
