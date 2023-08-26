//! A UI library for [Ambient](https://github.com/AmbientRun/Ambient). Built on top of [ambient_element](https://crates.io/crates/ambient_element).
//!
//! Ambient's UI system is heavily inspired by React (with hooks), and follows many of the same patterns.
//! Take a look at the [React documentation](https://react.dev/reference/react) to learn how hooks work in general.
//!
//! ## Getting started
//!
//! Here's a minimal, complete example of a counter app:
//!
//! ```ignore
//! use ambient_api::prelude::*;
//!
//! #[element_component]
//! fn App(hooks: &mut Hooks) -> Element {
//!     let (count, set_count) = hooks.use_state(0);
//!     FlowColumn::el([
//!         Text::el(format!("We've counted to {count} now")),
//!         Button::new("Increase", move |_| set_count(count + 1)).el(),
//!     ])
//! }
//!
//! #[main]
//! pub fn main() {
//!     App.el().spawn_interactive();
//! }
//! ```
//!
//! [See all UI examples here](https://github.com/AmbientRun/Ambient/tree/main/guest/rust/examples/ui).
#![deny(missing_docs)]

use ambient_cb::cb;
use ambient_element::{
    define_el_function_for_vec_element_newtype, element_component, to_owned, Element,
    ElementComponent, ElementComponentExt, Hooks, Setter,
};
use ambient_guest_bridge::{
    core::{
        app::components::{ui_scene, window_logical_size, window_physical_size},
        layout::components::{
            gpu_ui_size, height, margin, mesh_to_local_from_size, padding, width,
        },
        messages,
        rect::components::{background_color, background_url, rect},
        transform::components::{
            local_to_parent, local_to_world, mesh_to_local, mesh_to_world, scale, translation,
        },
    },
    ecs::World,
};
use ambient_shared_types::{ModifiersState, VirtualKeyCode};
use clickarea::ClickArea;
use glam::{vec3, Mat4, UVec2, Vec3, Vec4};

pub mod button;
pub mod clickarea;
pub mod default_theme;
pub mod dropdown;
pub mod editor;
pub mod layout;
pub mod prelude;
pub mod prompt;
pub mod screens;
pub mod scroll_area;
pub mod select;
pub mod tabs;
pub mod text;
pub mod throbber;
pub mod window;

/// A base element for all UI elements. It contains all the components needed for a UI element to work.
#[element_component]
pub fn UIBase(_: &mut Hooks) -> Element {
    Element::new()
        .init(translation(), vec3(0., 0., -0.001))
        .init_default(local_to_world())
        .init_default(local_to_parent())
        .init_default(mesh_to_world())
        .init(width(), 0.)
        .init(height(), 0.)
}

/// This only exists so that we can implement `From<String>` for [Text](text::Text), and then use it in
/// for instance [Button](button::Button).
pub struct UIElement(pub Element);
impl From<Element> for UIElement {
    fn from(el: Element) -> Self {
        Self(el)
    }
}

/// A simple UI rect. Use components like `width`, `height`, `background_color`, `border_color`, `border_radius` and `border_thickness`
/// to control its appearance.
#[element_component]
pub fn Rectangle(_hooks: &mut Hooks) -> Element {
    with_rect(UIBase.el())
        .with(width(), 100.)
        .with(height(), 100.)
        .with(background_color(), Vec4::ONE)
}

/// Converts the given element into a rect.
pub fn with_rect(element: Element) -> Element {
    element
        .init(rect(), ())
        .init(gpu_ui_size(), Vec4::ZERO)
        .init(mesh_to_local(), Mat4::IDENTITY)
        .init(scale(), Vec3::ONE)
        .init(mesh_to_local_from_size(), ())
        .init(ui_scene(), ())
}

/// Show an image loaded from a url
#[element_component]
pub fn ImageFromUrl(
    _: &mut Hooks,
    /// Url to load the image from
    url: String,
) -> Element {
    Rectangle
        .el()
        .with(background_color(), Vec4::ZERO)
        .with(background_url(), url)
}

/// A simple UI line. Use components like `line_from`, `line_to`, `line_width`, `background_color`, `border_color`, `border_radius` and `border_thickness`
/// to control its appearance.
#[element_component]
pub fn Line(_hooks: &mut Hooks) -> Element {
    with_rect(UIBase.el()).with(background_color(), Vec4::ONE)
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Keeps track of which element has focus.
pub struct Focus(Option<(String, u128)>);
impl Focus {
    /// Creates a new focus tracker.
    pub fn new(focus: Option<String>) -> Self {
        Self(focus.map(|x| (x, rand::random())))
    }
}

#[derive(Debug, Clone)]
/// Provides a context for focusable UI elements.
pub struct FocusRoot(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FocusRoot);
impl ElementComponent for FocusRoot {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        hooks.provide_context(|| Focus::new(None));
        let mut children = self.0;
        children.push(FocusResetter.el());
        Element::new().children(children)
    }
}
#[element_component]
fn FocusResetter(hooks: &mut Hooks) -> Element {
    let (focused, set_focus) = hooks.consume_context::<Focus>().unwrap();
    let (reset_focus, set_reset_focus) = hooks.use_state(Focus(None));
    hooks.use_runtime_message::<messages::WindowMouseInput>({
        to_owned![focused, set_reset_focus];
        move |_world, _event| {
            set_reset_focus(focused.clone());
        }
    });
    if focused == reset_focus && focused.0.is_some() {
        set_focus(Focus(None));
        set_reset_focus(Focus(None));
    }
    Element::new()
}

/// A trait that provides helper methods for UI elements.
pub trait UIExt {
    /// Wraps this element in a [ClickArea] element.
    fn with_clickarea(self) -> ClickArea;
    /// Adds a background color to this element.
    fn with_background(self, color: Vec4) -> Self;
    /// Adds padding to all sides of this element.
    fn with_padding_even(self, padding: f32) -> Self;
    /// Adds margin to all sides of this element.
    fn with_margin_even(self, margin: f32) -> Self;
}
impl UIExt for Element {
    fn with_clickarea(self) -> ClickArea {
        ClickArea::new(self)
    }
    fn with_background(self, background: Vec4) -> Self {
        with_rect(self).with(background_color(), background)
    }
    fn with_padding_even(self, value: f32) -> Self {
        self.with(padding(), Vec4::ONE * value)
    }
    fn with_margin_even(self, value: f32) -> Self {
        self.with(margin(), Vec4::ONE * value)
    }
}

/// A trait that provides helper hooks with UI-level functionality.
///
/// This should include all hooks that relate to game/UI state.
pub trait HooksExt {
    /// Helper wrapper around [Hooks::use_runtime_message] that listens to `WindowKeyboardInput` messages
    /// and parses them for you.
    ///
    /// The boolean is whether or not the button was pressed (true) or released (false).
    ///
    /// NOTE: This may be removed in future versions of the API when parsing is no longer necessary.
    fn use_keyboard_input(
        &mut self,
        func: impl Fn(&mut World, Option<VirtualKeyCode>, ModifiersState, bool) + Sync + Send + 'static,
    );

    /// A hook that returns the current window physical resolution (i.e. not taking DPI scaling into account)
    fn use_window_physical_resolution(&mut self) -> UVec2;
    /// A hook that returns the current window logical resolution (i.e. taking DPI scaling into account)
    fn use_window_logical_resolution(&mut self) -> UVec2;
    /// A hook that returns the current focus state for this element and a callback to set the focus state.
    fn use_focus(&mut self) -> (bool, Setter<bool>);
    /// A hook that returns the current focus state for this element, given a specific `instance_id`, and a callback to set the focus state.
    fn use_focus_for_instance_id(&mut self, instance_id: String) -> (bool, Setter<bool>);
}
impl HooksExt for Hooks<'_> {
    fn use_keyboard_input(
        &mut self,
        func: impl Fn(&mut World, Option<VirtualKeyCode>, ModifiersState, bool) + Sync + Send + 'static,
    ) {
        self.use_runtime_message(
            move |world, event: &ambient_guest_bridge::core::messages::WindowKeyboardInput| {
                func(
                    world,
                    event
                        .keycode
                        .as_ref()
                        .and_then(|k| k.parse::<VirtualKeyCode>().ok()),
                    ModifiersState::from_bits_truncate(event.modifiers),
                    event.pressed,
                );
            },
        );
    }

    // We need `clone` as resource is a ref on host and a copy on guest
    #[allow(clippy::clone_on_copy)]
    fn use_window_physical_resolution(&mut self) -> UVec2 {
        let (res, set_res) = self.use_state(self.world.resource(window_physical_size()).clone());
        self.use_frame(move |world| {
            let new_res = world.resource(window_physical_size()).clone();
            if new_res != res {
                set_res(new_res);
            }
        });
        res
    }

    // We need `clone` as resource is a ref on host and a copy on guest
    #[allow(clippy::clone_on_copy)]
    fn use_window_logical_resolution(&mut self) -> UVec2 {
        let (res, set_res) = self.use_state(self.world.resource(window_logical_size()).clone());
        self.use_frame(move |world| {
            let new_res = world.resource(window_logical_size()).clone();
            if new_res != res {
                set_res(new_res);
            }
        });
        res
    }

    fn use_focus(&mut self) -> (bool, Setter<bool>) {
        self.use_focus_for_instance_id(self.instance_id().to_owned())
    }

    fn use_focus_for_instance_id(&mut self, instance_id: String) -> (bool, Setter<bool>) {
        let (focus, set_focus) = self
            .consume_context::<Focus>()
            .expect("No FocusRoot available");
        let focused = if let Focus(Some((focused, _))) = &focus {
            focused == &instance_id
        } else {
            false
        };
        (
            focused,
            cb(move |new_focus| {
                set_focus(Focus::new(if new_focus {
                    Some(instance_id.clone())
                } else {
                    None
                }));
            }),
        )
    }
}
