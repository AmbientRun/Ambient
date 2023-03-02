use std::{
    collections::HashMap,
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
use ambient_input::{
    event_focus_change, event_mouse_input, event_mouse_motion, event_mouse_wheel,
    picking::{mouse_over, mouse_pickable},
};
pub use ambient_std::{cb, Cb};
use ambient_std::{color::Color, shapes::AABB};
use glam::*;
use itertools::Itertools;
use parking_lot::Mutex;
use winit::{
    event::{ElementState, ModifiersState, MouseButton, MouseScrollDelta},
    window::CursorGrabMode,
};

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
mod rect;
mod screens;
mod select;
mod style_constants;
mod tabs;
mod text_input;
mod throbber;

pub use ambient_layout as layout;
use ambient_text as text;
pub use ambient_text::*;
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
use rect::with_rect;
pub use rect::{background_color, border_color, border_radius, border_thickness, Corners, Rectangle};
pub use screens::*;
pub use select::*;
pub use style_constants::*;
pub use tabs::*;
pub use text_input::*;
pub use throbber::*;

pub use self::image::*;

pub fn init_all_componets() {
    layout::init_components();
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
pub struct WindowSized(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(WindowSized);
impl ElementComponent for WindowSized {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let res = use_window_logical_resolution(hooks);
        Dock(self.0).el().set(width(), res.x as _).set(height(), res.y as _).remove(local_to_parent())
    }
}

/// See https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#dock
#[derive(Debug, Clone)]
pub struct Dock(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Dock);
impl ElementComponent for Dock {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase).init(layout(), Layout::Dock).init_default(children()).children(self.0)
    }
}

/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct Flow(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Flow);
impl ElementComponent for Flow {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase).init(layout(), Layout::Flow).init_default(children()).children(self.0)
    }
}

/// A bookcase layout is a min-max layout; it should be a list of BookFiles, where each BookFile
/// has a `container` and a `book`. The book's determine the size of the entire Bookcase, but their
/// sizes are not manipulated. The containers are resized to fit the bookcase though, to aline them.
#[derive(Debug, Clone)]
pub struct Bookcase(pub Vec<BookFile>);
impl ElementComponent for Bookcase {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase)
            .init(layout(), Layout::Bookcase)
            .init_default(children())
            .children(self.0.into_iter().map(|x| x.el()).collect())
    }
}
#[derive(Debug, Clone)]
pub struct BookFile {
    container: Element,
    book: Element,
}
impl ElementComponent for BookFile {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase).init_default(is_book_file()).children(vec![self.container, self.book])
    }
}

/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct FlowColumn(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FlowColumn);
impl ElementComponent for FlowColumn {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .set(orientation(), Orientation::Vertical)
            .set(align_horizontal(), Align::Begin)
            .set(align_vertical(), Align::Begin)
            .set(fit_horizontal(), Fit::Children)
            .set(fit_vertical(), Fit::Children)
    }
}

/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct FlowRow(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FlowRow);
impl ElementComponent for FlowRow {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .set(orientation(), Orientation::Horizontal)
            .set(align_horizontal(), Align::Begin)
            .set(align_vertical(), Align::Begin)
            .set(fit_horizontal(), Fit::Children)
            .set(fit_vertical(), Fit::Children)
    }
}

#[derive(Debug, Clone)]
pub struct Centered(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Centered);
impl ElementComponent for Centered {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .set(orientation(), Orientation::Vertical)
            .set(align_horizontal(), Align::Center)
            .set(align_vertical(), Align::Center)
            .set(fit_horizontal(), Fit::None)
            .set(fit_vertical(), Fit::None)
    }
}

#[derive(Debug, Clone)]
pub struct ScrollArea(pub Element);
impl ElementComponent for ScrollArea {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (scroll, set_scroll) = hooks.use_state(0.);
        hooks.use_world_event(move |_world, event| {
            if let Some(delta) = event.get_ref(event_mouse_wheel()) {
                set_scroll(
                    scroll
                        + match delta {
                            MouseScrollDelta::LineDelta(_, y) => y * 20.,
                            MouseScrollDelta::PixelDelta(p) => p.y as f32,
                        },
                );
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

#[element_component]
pub fn FixedGrid(_: &mut Hooks, items: Vec<Element>, item_stride: Vec2, items_horizontal: usize) -> Element {
    UIBase.el().children(
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let x = i % items_horizontal;
                let y = i / items_horizontal;
                item.set(translation(), vec3(x as f32 * item_stride.x, y as f32 * item_stride.y, 0.))
            })
            .collect_vec(),
    )
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

pub trait UIExt {
    fn with_clickarea(self) -> ClickArea;
    fn with_background(self, color: Color) -> Self;
}
impl UIExt for Element {
    fn with_clickarea(self) -> ClickArea {
        ClickArea::new(self)
    }
    fn with_background(self, background: Color) -> Self {
        with_rect(self).set(background_color(), background)
    }
}

#[derive(Debug, Clone)]
pub struct ClickArea {
    pub inner: Element,
    pub on_mouse_enter: Vec<Cb<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub on_mouse_leave: Vec<Cb<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub on_mouse_hover: Vec<Cb<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub on_mouse_input: Vec<Cb<dyn Fn(&mut World, EntityId, ElementState, MouseButton) + Sync + Send>>,
    pub on_mouse_wheel: Vec<Cb<dyn Fn(&mut World, EntityId, MouseScrollDelta) + Sync + Send>>,
}
impl ClickArea {
    pub fn new(inner: Element) -> Self {
        Self {
            inner,
            on_mouse_enter: Vec::new(),
            on_mouse_leave: Vec::new(),
            on_mouse_hover: Vec::new(),
            on_mouse_input: Vec::new(),
            on_mouse_wheel: Vec::new(),
        }
    }
    pub fn on_mouse_hover<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_hover.push(cb(handle));
        self
    }
    pub fn on_mouse_enter<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_enter.push(cb(handle));
        self
    }
    pub fn on_mouse_leave<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_leave.push(cb(handle));
        self
    }
    pub fn on_mouse_input<F: Fn(&mut World, EntityId, ElementState, MouseButton) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_input.push(cb(handle));
        self
    }
    pub fn on_mouse_wheel<F: Fn(&mut World, EntityId, MouseScrollDelta) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_wheel.push(cb(handle));
        self
    }

    pub fn on_mouse_down<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.on_mouse_input(move |world, id, state, button| {
            if state == ElementState::Pressed {
                handle(world, id, button)
            }
        })
    }
    pub fn on_mouse_up<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.on_mouse_input(move |world, id, state, button| {
            if state == ElementState::Released {
                handle(world, id, button)
            }
        })
    }
}
impl ElementComponent for ClickArea {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { inner, on_mouse_enter, on_mouse_leave, on_mouse_hover, on_mouse_input, on_mouse_wheel } = *self;
        let id = hooks.use_ref_with(|_| None);
        let is_mouse_over = hooks.use_ref_with(|_| false);
        hooks.use_frame({
            let id = id.clone();
            let is_mouse_over = is_mouse_over.clone();
            move |world| {
                if let Some(id) = *id.lock() {
                    let next = world.get(id, mouse_over()).unwrap_or(false);
                    let mut state = is_mouse_over.lock();
                    if !*state && next {
                        for handler in &on_mouse_enter {
                            handler(world, id);
                        }
                    }
                    if *state && !next {
                        for handler in &on_mouse_leave {
                            handler(world, id);
                        }
                    }
                    if next {
                        for handler in &on_mouse_hover {
                            handler(world, id);
                        }
                    }
                    *state = next;
                }
            }
        });
        hooks.use_world_event({
            let id = id.clone();
            let is_mouse_over = is_mouse_over;
            move |world, event| {
                if let Some(id) = *id.lock() {
                    if let Some(event) = event.get_ref(event_mouse_input()) {
                        if *is_mouse_over.lock() {
                            for handler in &on_mouse_input {
                                handler(world, id, event.state, event.button);
                            }
                        }
                    } else if let Some(event) = event.get_ref(event_mouse_wheel()) {
                        if *is_mouse_over.lock() {
                            for handler in &on_mouse_wheel {
                                handler(world, id, *event);
                            }
                        }
                    }
                }
            }
        });
        inner.init(mouse_pickable(), AABB::ZERO).on_spawned(move |_, new_id| {
            *id.lock() = Some(new_id);
        })
    }
}

#[element_component]
pub fn MeasureSize(hooks: &mut Hooks, inner: Element, on_change: Cb<dyn Fn(Vec2) + Sync + Send + 'static>) -> Element {
    let (id, set_id) = hooks.use_state(None);
    let (current, set_current) = hooks.use_state(Vec2::ZERO);
    hooks.use_frame(move |world| {
        if let Some(id) = id {
            let width = world.get(id, width()).unwrap_or(0.);
            let height = world.get(id, height()).unwrap_or(0.);
            let next = vec2(width, height);
            if current != next {
                on_change(next);
                set_current(next);
            }
        }
    });
    inner.on_spawned(move |_, id| set_id(Some(id)))
}

#[derive(Debug, Clone)]
pub struct TransformGroup(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(TransformGroup);
impl ElementComponent for TransformGroup {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        Element::new()
            .set_default(local_to_world())
            .children(self.0.into_iter().map(|x| Element::from(TransformGroupChild(x)).init_default(local_to_parent())).collect_vec())
    }
}

#[derive(Debug, Clone)]
struct TransformGroupChild(Element);
impl ElementComponent for TransformGroupChild {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct TransformMapGroup(pub HashMap<String, Element>);
impl ElementComponent for TransformMapGroup {
    fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
        TransformGroup(self.0.into_iter().sorted_by_key(|x| x.0.clone()).map(|(k, v)| v.key(k)).collect()).into()
    }
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
    let el = Flow(vec![]).el().with_background(Color::rgba(0., 0., 0., 0.8));
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
