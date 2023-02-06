#![feature(let_chains)]
#[macro_use]
extern crate closure;

use std::{
    collections::HashMap, fmt::Debug, ops::Deref, sync::{
        atomic::{AtomicBool, Ordering}, Arc
    }
};

use elements_core::{hierarchy::children, on_frame, on_window_event, transform::*, window, window_logical_size, window_physical_size};
pub use elements_ecs::{EntityId, SystemGroup, World};
pub use elements_editor_derive::ElementEditor;
pub use elements_element as element;
use elements_element::{
    define_el_function_for_vec_element_newtype, element_component, Element, ElementComponent, ElementComponentExt, Hooks
};
use elements_input::{
    on_app_mouse_input, on_app_mouse_motion, on_app_mouse_wheel, picking::{mouse_pickable, on_mouse_enter, on_mouse_hover, on_mouse_input, on_mouse_leave, on_mouse_wheel}
};
pub use elements_std::Cb;
use elements_std::{color::Color, shapes::AABB, time::Timeout};
use glam::*;
use itertools::Itertools;
use parking_lot::Mutex;
use winit::event::{ElementState, ModifiersState, MouseButton, MouseScrollDelta, WindowEvent};

mod asset_url;
mod button;
mod collections;
mod dropdown;
pub mod graph;
mod hooks;
mod image;
mod input;
pub mod layout;
mod loadable;
mod prompt;
mod rect;
mod screens;
mod select;
mod style_constants;
mod tabs;
mod text;
mod text_input;
mod text_material;
mod throbber;

pub use asset_url::*;
pub use button::*;
pub use collections::*;
pub use dropdown::*;
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
pub use text::*;
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

/// This only exists so that we can implement From<String> for Text, and then use it in
/// for instance Button
pub struct UIElement(pub Element);
impl From<Element> for UIElement {
    fn from(el: Element) -> Self {
        Self(el)
    }
}

#[element_component]
pub fn UIBase(_: &mut World, _: &mut Hooks) -> Element {
    Element::new()
        .init(translation(), vec3(0., 0., -0.001))
        .init_default(local_to_world())
        .init_default(local_to_parent())
        .init_default(mesh_to_world())
        .init(width(), 0.)
        .init(height(), 0.)
}

pub fn use_window_physical_resolution(world: &World, hooks: &mut Hooks) -> UVec2 {
    let (res, set_res) = hooks.use_state(*world.resource(window_physical_size()));
    hooks.use_frame(move |world| {
        let new_res = *world.resource(window_physical_size());
        if new_res != res {
            set_res(new_res);
        }
    });
    res
}
pub fn use_window_logical_resolution(world: &World, hooks: &mut Hooks) -> UVec2 {
    let (res, set_res) = hooks.use_state(*world.resource(window_logical_size()));
    hooks.use_frame(move |world| {
        let new_res = *world.resource(window_logical_size());
        if new_res != res {
            set_res(new_res);
        }
    });
    res
}

#[derive(Debug, Clone)]
pub struct WindowSized(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(WindowSized);
impl ElementComponent for WindowSized {
    fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
        let res = use_window_logical_resolution(world, hooks);
        Dock(self.0).el().set(width(), res.x as _).set(height(), res.y as _).remove(local_to_parent())
    }
}

/// See https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#dock
#[derive(Debug, Clone)]
pub struct Dock(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Dock);
impl ElementComponent for Dock {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        Element::from(UIBase).init(layout(), Layout::Dock).init_default(children()).children(self.0)
    }
}

/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct Flow(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Flow);
impl ElementComponent for Flow {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        Element::from(UIBase).init(layout(), Layout::Flow).init_default(children()).children(self.0)
    }
}

/// A bookcase layout is a min-max layout; it should be a list of BookFiles, where each BookFile
/// has a `container` and a `book`. The book's determine the size of the entire Bookcase, but their
/// sizes are not manipulated. The containers are resized to fit the bookcase though, to aline them.
#[derive(Debug, Clone)]
pub struct Bookcase(pub Vec<BookFile>);
impl ElementComponent for Bookcase {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
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
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        Element::from(UIBase).init_default(is_book_file()).children(vec![self.container, self.book])
    }
}

/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct FlowColumn(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FlowColumn);
impl ElementComponent for FlowColumn {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
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
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
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
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
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
    fn render(self: Box<Self>, _: &mut World, hooks: &mut Hooks) -> Element {
        let (scroll, set_scroll) = hooks.use_state(0.);
        UIBase
            .el()
            .init_default(children())
            .children(vec![
                // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                Flow(vec![self.0]).el().set(fit_horizontal(), Fit::Parent).set(translation(), vec3(0., scroll, 0.)),
            ])
            .set(layout(), Layout::WidthToChildren)
            .listener(
                on_app_mouse_wheel(),
                Arc::new(move |_, _, delta| {
                    set_scroll(
                        scroll
                            + match delta {
                                MouseScrollDelta::LineDelta(_, y) => y * 20.,
                                MouseScrollDelta::PixelDelta(p) => p.y as f32,
                            },
                    );
                    true
                }),
            )
    }
}
impl ScrollArea {
    pub fn el(element: Element) -> Element {
        Self(element).el()
    }
}

#[element_component]
pub fn FixedGrid(_: &mut World, _: &mut Hooks, items: Vec<Element>, item_stride: Vec2, items_horizontal: usize) -> Element {
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
    fn render(self: Box<Self>, _: &mut World, hooks: &mut Hooks) -> Element {
        let set_focus = hooks.provide_context(|| Focus(None));
        Element::new().listener(on_app_mouse_input(), Arc::new(move |_, _, _| set_focus(Focus(None)))).children(self.0)
    }
}

impl Default for HighjackMouse {
    fn default() -> Self {
        Self { on_mouse_move: Cb::new(|_, _, _| {}), on_click: Cb::new(|_| {}), hide_mouse: false }
    }
}

#[element_component]
pub fn HighjackMouse(
    _: &mut World,
    hooks: &mut Hooks,
    on_mouse_move: Cb<dyn Fn(&World, Vec2, Vec2) + Sync + Send>,
    on_click: Cb<dyn Fn(MouseButton) + Sync + Send>,
    hide_mouse: bool,
) -> Element {
    // let (window_focused, _) = hooks.use_state(Arc::new(AtomicBool::new(true)));
    // Assume window has focus
    let focused = Arc::new(AtomicBool::new(true));
    let position = hooks.use_ref_with(|| Vec2::ZERO);
    hooks.use_spawn(move |world| {
        if hide_mouse {
            world.resource(window()).set_cursor_grab(winit::window::CursorGrabMode::Locked).ok();
            world.resource(window()).set_cursor_visible(false);
        }
        Box::new(move |world| {
            let window = world.resource(window());
            if hide_mouse {
                window.set_cursor_grab(winit::window::CursorGrabMode::None).ok();
                window.set_cursor_visible(true);
            }
        })
    });
    WindowSized(vec![])
        .el()
        .on_mouse_down(move |_, _, button| on_click(button))
        .set(translation(), -Vec3::Z * 0.99)
        .listener(
            on_app_mouse_motion(),
            Arc::new(closure!(clone focused, |world, _, delta| {
                let pos = {
                    let mut pos = position.lock();
                    *pos += delta;
                    *pos
                };

                if focused.load(Ordering::Relaxed) {
                    on_mouse_move(world, pos, delta);
                }
            })),
        )
        .listener(
            on_window_event(),
            Arc::new(move |w, _, event| {
                if let WindowEvent::Focused(f) = event {
                    let window = w.resource(window());
                    window.set_cursor_visible(!f);
                    // Fails on android/IOS
                    window
                        .set_cursor_grab(if *f { winit::window::CursorGrabMode::Locked } else { winit::window::CursorGrabMode::None })
                        .ok();
                    focused.store(*f, Ordering::Relaxed);
                }
            }),
        )
}

#[derive(Clone, Debug)]
pub struct EditorOpts {
    pub enum_can_change_type: bool,
}

impl Default for EditorOpts {
    fn default() -> Self {
        Self { enum_can_change_type: true }
    }
}

pub trait Editor {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element;
}

impl Editor for EditableDuration {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        DurationEditor::new(value, on_change.unwrap_or_else(|| Cb(Arc::new(|_| {})))).el()
    }
}

impl<T: Editor + 'static> Editor for Box<T> {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        T::editor(
            *value,
            on_change.map(|cb| Cb(Arc::new(move |new_value| cb.0(Box::new(new_value))) as Arc<dyn Fn(T) + Sync + Send>)),
            opts,
        )
    }
}

impl<T> Editor for Arc<T>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        T::editor(value.deref().clone(), on_change.map(|f| Cb::new(move |v: T| f(Arc::new(v))) as Cb<dyn Fn(T) + Sync + Send>), opts)
    }
}

impl<T> Editor for Arc<Mutex<T>>
where
    T: 'static + Send + Sync + Clone + Editor,
{
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        let v: T = value.lock().clone();
        T::editor(
            v,
            on_change.map(|f| {
                Cb::new(move |v: T| {
                    // Update the shared value
                    *value.lock() = v;
                    // Give the same value which is now internally mutated
                    f(value.clone())
                }) as Cb<dyn Fn(T) + Sync + Send>
            }),
            opts,
        )
    }
}
impl Editor for () {
    fn editor(_value: Self, _on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _opts: EditorOpts) -> Element {
        Element::new()
    }
}

impl Editor for Timeout {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _: EditorOpts) -> Element {
        let on_change = on_change.unwrap_or_else(|| Cb::new(|_| {}));

        DurationEditor::new(
            EditableDuration::new(value.duration(), true, value.duration().as_secs().to_string()),
            Cb::new(move |v| (on_change)(Timeout::new(v.dur()))),
        )
        .el()
    }
}

impl<T: Default + Editor + 'static> Editor for Option<T> {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: EditorOpts) -> Element {
        if let Some(on_change) = on_change {
            if let Some(inner_value) = value {
                FlowRow(vec![
                    Button::new("\u{f056}", closure!(clone on_change, |_| on_change.0(None))).style(ButtonStyle::Flat).el(),
                    T::editor(inner_value, Some(Cb(Arc::new(closure!(clone on_change, |value| on_change.0(Some(value)))))), opts),
                ])
                .el()
            } else {
                Button::new("\u{f055}", closure!(clone on_change, |_| on_change.0(Some(T::default())))).style(ButtonStyle::Flat).el()
            }
        } else if let Some(value) = value {
            T::editor(value, None, opts)
        } else {
            Text::el("None")
        }
    }
}

pub trait UIExt {
    fn on_mouse_hover<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn on_mouse_enter<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn on_mouse_leave<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn on_mouse_wheel<F: Fn(&mut World, EntityId, MouseScrollDelta) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn on_mouse_input<F: Fn(&mut World, EntityId, ElementState, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn on_mouse_down<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn on_mouse_up<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self;
    fn with_clickarea(self) -> Self;
    fn with_background(self, color: Color) -> Self;
    fn on_size_change(self, current: Vec2, on_change: Arc<dyn Fn(Vec2) + Sync + Send + 'static>) -> Self;
}
impl UIExt for Element {
    fn on_mouse_hover<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.with_clickarea().listener(on_mouse_hover(), Arc::new(handle))
    }
    fn on_mouse_enter<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.with_clickarea().listener(on_mouse_enter(), Arc::new(handle))
    }
    fn on_mouse_leave<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.with_clickarea().listener(on_mouse_leave(), Arc::new(handle))
    }
    fn on_mouse_wheel<F: Fn(&mut World, EntityId, MouseScrollDelta) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.with_clickarea().listener(on_mouse_wheel(), Arc::new(handle))
    }
    fn on_mouse_input<F: Fn(&mut World, EntityId, ElementState, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.with_clickarea().listener(on_mouse_input(), Arc::new(handle))
    }
    fn on_mouse_down<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.on_mouse_input(move |world, id, state, button| {
            if state == ElementState::Pressed {
                handle(world, id, button)
            }
        })
    }
    fn on_mouse_up<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.on_mouse_input(move |world, id, state, button| {
            if state == ElementState::Released {
                handle(world, id, button)
            }
        })
    }
    fn with_clickarea(self) -> Self {
        self.init(mouse_pickable(), AABB::ZERO)
    }
    fn with_background(self, background: Color) -> Self {
        with_rect(self).set(background_color(), background)
    }
    fn on_size_change(self, current: Vec2, on_change: Arc<dyn Fn(Vec2) + Sync + Send + 'static>) -> Self {
        self.listener(
            on_frame(),
            Arc::new(move |world, id, _| {
                let width = world.get(id, width()).unwrap_or(0.);
                let height = world.get(id, height()).unwrap_or(0.);
                if current.x != width || current.y != height {
                    on_change(vec2(width, height));
                }
            }),
        )
    }
}

#[derive(Debug, Clone)]
pub struct TransformGroup(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(TransformGroup);
impl ElementComponent for TransformGroup {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        Element::new()
            .set_default(local_to_world())
            .children(self.0.into_iter().map(|x| Element::from(TransformGroupChild(x)).init_default(local_to_parent())).collect_vec())
    }
}

#[derive(Debug, Clone)]
struct TransformGroupChild(Element);
impl ElementComponent for TransformGroupChild {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct TransformMapGroup(pub HashMap<String, Element>);
impl ElementComponent for TransformMapGroup {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
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
pub fn FontAwesomeIcon(_world: &mut World, _hooks: &mut Hooks, icon: u32, solid: bool) -> Element {
    Text::el(char::from_u32(icon).unwrap().to_string()).set(font_family(), FontFamily::FontAwesome { solid })
}

#[element_component]
pub fn Separator(_world: &mut World, _hooks: &mut Hooks, vertical: bool) -> Element {
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
        set_screen: Arc<dyn Fn(Option<Element>) + Send + Sync>,
    ) -> impl Fn() + Sync + Send {
        let value = self.clone();
        let title = title.into();
        move || {
            set_screen(Some(
                EditorPrompt {
                    title: title.clone(),
                    value: value.get_cloned(),
                    set_screen: Cb(set_screen.clone()),
                    on_ok: Cb::new({
                        let value = value.clone();
                        move |_, new_value| value.set(new_value)
                    }),
                    on_cancel: Some(Cb::new(|_| {})),
                    validator: None,
                }
                .el(),
            ));
        }
    }
}
