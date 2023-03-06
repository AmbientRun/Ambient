use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ambient_core::{hierarchy::children, transform::*, window::WindowCtl, window_ctl};
pub use ambient_ecs::{EntityId, SystemGroup, World};
pub use ambient_editor_derive::ElementEditor;
pub use ambient_element as element;
use ambient_element::{
    define_el_function_for_vec_element_newtype, element_component, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_input::{event_focus_change, event_mouse_input, event_mouse_motion, event_mouse_wheel, event_mouse_wheel_pixels};
use ambient_std::color::Color;
pub use ambient_std::{cb, Cb};
use glam::*;
use parking_lot::Mutex;
use winit::{event::ModifiersState, window::CursorGrabMode};

mod asset_url;
mod button;
mod collections;
mod dropdown;
mod editor;
pub mod graph;
mod hooks;
mod image;
mod input;
mod loadable;
mod prompt;

mod screens;
mod select;
mod style_constants;
mod tabs;
mod text_input;
mod throbber;

pub use ambient_layout as layout;
pub use ambient_rect as rect;
pub use ambient_rect::{background_color, border_color, border_radius, border_thickness, Corners};
use ambient_text as text;
pub use ambient_text::*;
pub use ambient_ui_components::clickarea::*;
pub use ambient_ui_components::layout::*;
pub use ambient_ui_components::text::*;
pub use ambient_ui_components::*;
pub use asset_url::*;
pub use button::*;
pub use collections::*;
pub use dropdown::*;
pub use editor::*;
pub use hooks::*;
pub use input::*;
pub use layout::*;
pub use loadable::*;
pub use prompt::*;
pub use screens::*;
pub use select::*;
pub use style_constants::*;
pub use tabs::*;
pub use text_input::*;
pub use throbber::*;

pub use self::image::*;

pub fn init_all_componets() {
    layout::init_all_components();
    layout::init_gpu_components();
    rect::init_components();
    text::init_components();
    screens::init_components();
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui",
        vec![Box::new(rect::systems()), Box::new(text::systems()), Box::new(layout::layout_systems()), Box::new(screens::systems())],
    )
}

#[derive(Debug, Clone)]
pub struct ScrollArea(pub Element);
impl ElementComponent for ScrollArea {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (scroll, set_scroll) = hooks.use_state(0.);
        hooks.use_world_event(move |_world, event| {
            if let Some(delta) = event.get(event_mouse_wheel()) {
                set_scroll(scroll + if event.get(event_mouse_wheel_pixels()).unwrap() { delta.y } else { delta.y * 20. });
            }
        });
        UIBase
            .el()
            .init_default(children())
            .children(vec![
                // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                Flow(vec![self.0]).el().set(fit_horizontal(), Fit::Parent).set(translation(), vec3(0., scroll, 0.)),
            ])
            .set(layout(), Layout::WidthToChildren)
    }
}
impl ScrollArea {
    pub fn el(element: Element) -> Element {
        Self(element).el()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Focus(Option<EntityId>);

pub fn use_has_focus(_: &World, hooks: &mut Hooks) -> bool {
    hooks.consume_context::<Focus>().is_some()
}

#[derive(Debug, Clone)]
/// Provides a context for focusable UI elements
pub struct FocusRoot(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FocusRoot);
impl ElementComponent for FocusRoot {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let set_focus = hooks.provide_context(|| Focus(None));
        hooks.use_world_event(move |_world, event| {
            if let Some(_event) = event.get_ref(event_mouse_input()) {
                set_focus(Focus(None));
            }
        });
        Element::new().children(self.0)
    }
}

impl Default for HighjackMouse {
    fn default() -> Self {
        Self { on_mouse_move: cb(|_, _, _| {}), on_click: cb(|_| {}), hide_mouse: false }
    }
}

#[element_component]
pub fn HighjackMouse(
    hooks: &mut Hooks,
    on_mouse_move: Cb<dyn Fn(&World, Vec2, Vec2) + Sync + Send>,
    on_click: Cb<dyn Fn(MouseButton) + Sync + Send>,
    hide_mouse: bool,
) -> Element {
    // let (window_focused, _) = hooks.use_state(Arc::new(AtomicBool::new(true)));
    // Assume window has focus
    let focused = Arc::new(AtomicBool::new(true));
    let position = hooks.use_ref_with(|_| Vec2::ZERO);
    hooks.use_spawn(move |world| {
        if hide_mouse {
            let ctl = world.resource(window_ctl());
            ctl.send(WindowCtl::GrabCursor(CursorGrabMode::Locked)).ok();
            ctl.send(WindowCtl::ShowCursor(false)).ok();
        }
        Box::new(move |world| {
            if hide_mouse {
                let ctl = world.resource(window_ctl());
                ctl.send(WindowCtl::GrabCursor(CursorGrabMode::None)).ok();
                ctl.send(WindowCtl::ShowCursor(true)).ok();
            }
        })
    });
    hooks.use_world_event({
        let focused = focused;
        move |world, event| {
            if let Some(delta) = event.get_ref(event_mouse_motion()) {
                let pos = {
                    let mut pos = position.lock();
                    *pos += *delta;
                    *pos
                };

                if focused.load(Ordering::Relaxed) {
                    on_mouse_move(world, pos, *delta);
                }
            } else if let Some(f) = event.get(event_focus_change()) {
                let ctl = world.resource(window_ctl());
                ctl.send(WindowCtl::ShowCursor(!f)).ok();
                // window.set_cursor_visible(!f);
                // Fails on android/IOS
                ctl.send(WindowCtl::GrabCursor(if f {
                    winit::window::CursorGrabMode::Locked
                } else {
                    winit::window::CursorGrabMode::None
                }))
                .ok();

                focused.store(f, Ordering::Relaxed);
            }
        }
    });
    WindowSized(vec![]).el().with_clickarea().on_mouse_down(move |_, _, button| on_click(button)).el().set(translation(), -Vec3::Z * 0.99)
}

/// Ctrl on windows, Command on osx
pub fn command_modifier() -> ModifiersState {
    #[cfg(target_os = "macos")]
    return ModifiersState::LOGO;
    #[cfg(not(target_os = "macos"))]
    return ModifiersState::CTRL;
}

#[element_component]
pub fn FontAwesomeIcon(_hooks: &mut Hooks, icon: u32, solid: bool) -> Element {
    Text::el(char::from_u32(icon).unwrap().to_string()).set(font_family(), FontFamily::FontAwesome { solid })
}

#[element_component]
pub fn Separator(_hooks: &mut Hooks, vertical: bool) -> Element {
    let el = Flow(vec![]).el().with_background(Color::rgba(0., 0., 0., 0.8).into());
    if vertical {
        el.set(width(), 1.).set(fit_horizontal(), Fit::None).set(fit_vertical(), Fit::Parent)
    } else {
        el.set(height(), 1.).set(fit_horizontal(), Fit::Parent).set(fit_vertical(), Fit::None)
    }
}

#[derive(Clone)]
/// Helper for mutating UI state in multiple places.
pub struct WithChange<T: Clone>(Arc<Mutex<T>>, Cb<dyn Fn(T) + Sync + Send>);
impl<T: Clone> WithChange<T> {
    pub fn new(value: &T, changer: &Cb<dyn Fn(T) + Sync + Send>) -> Self {
        Self(Arc::new(Mutex::new(value.clone())), changer.clone())
    }

    pub fn change(&self, changer: impl FnOnce(&mut T)) {
        let mut lock = self.0.lock();
        changer(&mut lock);
        self.1(lock.clone());
    }

    pub fn query<R>(&self, extractor: impl FnOnce(&T) -> R) -> R {
        extractor(&self.0.lock())
    }
}

#[derive(Clone)]
/// Helper that takes an existing `WithChange<T>` and applies a projection from `T` to `U`.
///
/// That is, this allows you to mutate a field of the `T` in isolation.
pub struct WithChangePart<T: Clone, U: Clone, F: Fn(&mut T) -> &mut U>(WithChange<T>, F);
impl<T: Clone, U: Clone, F: Fn(&mut T) -> &mut U> WithChangePart<T, U, F> {
    pub fn new(with_change: WithChange<T>, projection: F) -> Self {
        Self(with_change, projection)
    }

    pub fn change(&self, changer: impl FnOnce(&mut U)) {
        self.0.change(|value| changer(self.1(value)));
    }

    pub fn query<R>(&self, extractor: impl FnOnce(&U) -> R) -> R {
        extractor(self.1(&mut self.0 .0.lock()))
    }

    pub fn get_cloned(&self) -> U {
        self.query(|value| value.clone())
    }

    pub fn set(&self, value: U) {
        self.change(|r| *r = value);
    }
}
impl<
        T: Clone + Send + Sync + 'static,
        U: Clone + Editor + Debug + Send + Sync + 'static,
        F: Fn(&mut T) -> &mut U + Clone + Send + Sync + 'static,
    > WithChangePart<T, U, F>
{
    /// Helper method that generates a callback that, when called, sets the current
    /// screen to an EditorPrompt that edits this value.
    pub fn to_editor_prompt_screen_callback(
        &self,
        title: impl Into<String>,
        set_screen: Cb<dyn Fn(Option<Element>) + Send + Sync>,
    ) -> impl Fn() + Sync + Send {
        let value = self.clone();
        let title = title.into();
        move || {
            set_screen(Some(
                EditorPrompt {
                    title: title.clone(),
                    value: value.get_cloned(),
                    set_screen: set_screen.clone(),
                    on_ok: cb({
                        let value = value.clone();
                        move |_, new_value| value.set(new_value)
                    }),
                    on_cancel: Some(cb(|_| {})),
                    validator: None,
                }
                .el(),
            ));
        }
    }
}
