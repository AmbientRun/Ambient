use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ambient_core::{transform::*, window::window_ctl, window::WindowCtl};
pub use ambient_ecs::{EntityId, SystemGroup, World};
pub use ambient_editor_derive::ElementEditor;
pub use ambient_element as element;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_input::{event_focus_change, event_mouse_motion};
pub use ambient_std::{cb, Cb};
use ambient_window_types::ModifiersState;
use glam::*;
use parking_lot::Mutex;
use winit::window::CursorGrabMode;

// mod asset_url;

mod component_editor;
pub mod graph;
mod hooks;
mod image;

pub use ambient_layout as layout;
pub use ambient_rect as rect;
pub use ambient_rect::{background_color, border_color, border_radius, border_thickness, Corners};
use ambient_text as text;
pub use ambient_text::*;
pub use ambient_ui_components::clickarea::*;
pub use ambient_ui_components::default_theme as style_constants;
pub use ambient_ui_components::*;
pub use ambient_ui_components::{button, dropdown, prompt, select, tabs, throbber};
pub use ambient_ui_components::{editor::*, layout::*, scroll_area::*, text::*};
// pub use asset_url::*;
pub use button::*;
pub use component_editor::*;
pub use dropdown::*;
pub use editor::*;
pub use hooks::*;
pub use layout::*;
pub use prompt::*;
pub use screens::*;
pub use select::*;
pub use style_constants::*;
pub use tabs::*;
pub use throbber::*;

pub use self::image::*;
use ambient_event_types::{WINDOW_FOCUSED, WINDOW_MOUSE_MOTION};
use ambient_window_types::MouseButton;

pub fn init_all_components() {
    layout::init_all_components();
    layout::init_gpu_components();
    rect::init_components();
    text::init_components();
}

pub fn systems() -> SystemGroup {
    SystemGroup::new("ui", vec![Box::new(rect::systems()), Box::new(text::systems(true)), Box::new(layout::layout_systems())])
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
    hooks.use_multi_event(&[WINDOW_MOUSE_MOTION, WINDOW_FOCUSED], {
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
