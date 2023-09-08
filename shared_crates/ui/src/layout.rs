//! Implements all of the [Element]s used for layouting.
//!
//! The layout is roughly based on [Windows Forms](https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout).
//!
//! There are two major layout components, [Dock] and [Flow] (which includes [FlowColumn] and [FlowRow]).
use crate::{use_window_logical_resolution, UIBase, UIExt};
use ambient_cb::Cb;
use ambient_color::Color;
use ambient_element::{
    define_el_function_for_vec_element_newtype, element_component, use_frame, use_state, Element,
    ElementComponent, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::{
    ecs::components::children,
    transform::components::{local_to_parent, local_to_world, translation},
};
use glam::{vec2, vec3, Mat4, Vec2, Vec3};
use itertools::Itertools;

pub use ambient_guest_bridge::core::layout::{
    components::*,
    types::{Align, Docking, Fit, Layout, Orientation},
};

#[derive(Debug, Clone)]
/// A [Dock] that is always the size of the window.
pub struct WindowSized(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(WindowSized);
impl ElementComponent for WindowSized {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let res = use_window_logical_resolution(hooks);
        Dock(self.0)
            .el()
            .with(width(), res.x as _)
            .with(height(), res.y as _)
            .remove(local_to_parent())
    }
}

/// A docking layout, where each child specifies which side of the parent it should be docked to.
/// It is top-down: it starts with a given area (say the screen) and then divides it into smaller pieces with each new element added to it.
///
/// The child specifies which side to dock to using the `docking_` components.
///
/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#dock>.
#[derive(Debug, Clone)]
pub struct Dock(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Dock);
impl ElementComponent for Dock {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase)
            .init(layout(), Layout::Dock)
            .init_default(children())
            .children(self.0)
    }
}

/// A flow layout.
/// It is bottom-up: it auto-resizes itself to fit its constituent components.
///
/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>.
#[derive(Debug, Clone)]
pub struct Flow(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Flow);
impl ElementComponent for Flow {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase)
            .init(layout(), Layout::Flow)
            .init_default(children())
            .children(self.0)
    }
}

/// A bookcase layout is a min-max layout; it should be a list of [BookFile]s, where each [BookFile]
/// has a `container` and a `book`. The book's determine the size of the entire [Bookcase], but their
/// sizes are not manipulated. The containers are resized to fit the bookcase though, to align them.
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
/// An entry in a [Bookcase].
pub struct BookFile {
    /// The container for the book.
    container: Element,
    /// The book itself.
    book: Element,
}
impl ElementComponent for BookFile {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase)
            .init_default(is_book_file())
            .children(vec![self.container, self.book])
    }
}

/// A [FlowColumn] is a [Flow] that is oriented vertically.
///
/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct FlowColumn(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FlowColumn);
impl ElementComponent for FlowColumn {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .with(orientation(), Orientation::Vertical)
            .with(align_horizontal(), Align::Begin)
            .with(align_vertical(), Align::Begin)
            .with(fit_horizontal(), Fit::Children)
            .with(fit_vertical(), Fit::Children)
    }
}

//// A [FlowRow] is a [Flow] that is oriented horizontally.
///
/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct FlowRow(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(FlowRow);
impl ElementComponent for FlowRow {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .with(orientation(), Orientation::Horizontal)
            .with(align_horizontal(), Align::Begin)
            .with(align_vertical(), Align::Begin)
            .with(fit_horizontal(), Fit::Children)
            .with(fit_vertical(), Fit::Children)
    }
}

/// A [Centered] is a [Flow] that is oriented vertically and is centered.
///
#[derive(Debug, Clone)]
pub struct Centered(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Centered);
impl ElementComponent for Centered {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .with(orientation(), Orientation::Vertical)
            .with(align_horizontal(), Align::Center)
            .with(align_vertical(), Align::Center)
            .with(fit_horizontal(), Fit::None)
            .with(fit_vertical(), Fit::None)
    }
}

/// A [FixedGrid] is a grid of elements with a fixed stride.
#[element_component]
pub fn FixedGrid(
    _: &mut Hooks,
    /// The items to put in the grid. Must be a multiple of `items_horizontal`. (i.e. a 2D array represented as a 1D array)
    items: Vec<Element>,
    /// The display stride between items (i.e. how much space each item has).
    item_stride: Vec2,
    /// The number of items in a row.
    items_horizontal: usize,
) -> Element {
    UIBase.el().children(
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let x = i % items_horizontal;
                let y = i / items_horizontal;
                item.with(
                    translation(),
                    vec3(x as f32 * item_stride.x, y as f32 * item_stride.y, 0.),
                )
            })
            .collect_vec(),
    )
}

/// Measures the size of its inner element and calls the callback when it changes.
#[element_component]
pub fn MeasureSize(
    hooks: &mut Hooks,
    /// The element to measure.
    inner: Element,
    /// The callback to call when the size changes.
    on_change: Cb<dyn Fn(Vec2) + Sync + Send + 'static>,
) -> Element {
    let (id, set_id) = use_state(hooks, None);
    let (current, set_current) = use_state(hooks, Vec2::ZERO);

    use_frame(hooks, move |world| {
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
    inner.on_spawned(move |_, id, _| set_id(Some(id)))
}

/// Measures the absolute position of its inner element and calls the callback when it changes.
#[element_component]
pub fn MeasureAbsolutePosition(
    hooks: &mut Hooks,
    /// The element to measure.
    inner: Element,
    /// The callback to call when the absolute position changes.
    on_change: Cb<dyn Fn(Vec3) + Sync + Send + 'static>,
) -> Element {
    let (id, set_id) = use_state(hooks, None);
    let (current, set_current) = use_state(hooks, Vec3::ZERO);
    use_frame(hooks, move |world| {
        if let Some(id) = id {
            let ltw = world.get(id, local_to_world()).unwrap();
            let (_, _, abs_pos) = Mat4::to_scale_rotation_translation(&ltw);
            if current != abs_pos {
                on_change(abs_pos);
                set_current(abs_pos);
            }
        }
    });
    inner.on_spawned(move |_, id, _| set_id(Some(id)))
}

#[element_component]
/// A simple separator, similar to `<hr>` in HTML.
pub fn Separator(
    _hooks: &mut Hooks,
    /// Whether the separator is vertical or horizontal.
    vertical: bool,
) -> Element {
    let el = Flow(vec![])
        .el()
        .with_background(Color::rgba(0., 0., 0., 0.8).into());
    if vertical {
        el.with(width(), 1.)
            .with(fit_horizontal(), Fit::None)
            .with(fit_vertical(), Fit::Parent)
    } else {
        el.with(height(), 1.)
            .with(fit_horizontal(), Fit::Parent)
            .with(fit_vertical(), Fit::None)
    }
}
