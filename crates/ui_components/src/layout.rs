use crate::{use_window_logical_resolution, UIBase};
use ambient_cb::Cb;
use ambient_element::{
    define_el_function_for_vec_element_newtype, element_component, Element, ElementComponent, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::components::{
    ecs::children,
    layout::{
        align_horizontal_begin, align_horizontal_center, align_vertical_begin, align_vertical_center, fit_horizontal_children,
        fit_horizontal_none, fit_vertical_children, fit_vertical_none, height, is_book_file, layout_bookcase, layout_dock, layout_flow,
        orientation_horizontal, orientation_vertical, width,
    },
    transform::{local_to_parent, translation},
};
use glam::{vec2, vec3, Vec2};
use itertools::Itertools;

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
        Element::from(UIBase).init_default(layout_dock()).init_default(children()).children(self.0)
    }
}

/// See <https://docs.microsoft.com/en-us/dotnet/desktop/winforms/controls/layout?view=netdesktop-6.0#container-flow-layout>
#[derive(Debug, Clone)]
pub struct Flow(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Flow);
impl ElementComponent for Flow {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Element::from(UIBase).init_default(layout_flow()).init_default(children()).children(self.0)
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
            .init_default(layout_bookcase())
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
            .set_default(orientation_vertical())
            .set_default(align_horizontal_begin())
            .set_default(align_vertical_begin())
            .set_default(fit_horizontal_children())
            .set_default(fit_vertical_children())
        // .set(orientation(), Orientation::Vertical)
        // .set(align_horizontal(), Align::Begin)
        // .set(align_vertical(), Align::Begin)
        // .set(fit_horizontal(), Fit::Children)
        // .set(fit_vertical(), Fit::Children)
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
            .set_default(orientation_horizontal())
            .set_default(align_horizontal_begin())
            .set_default(align_vertical_begin())
            .set_default(fit_horizontal_children())
            .set_default(fit_vertical_children())
        // .set(orientation(), Orientation::Horizontal)
        // .set(align_horizontal(), Align::Begin)
        // .set(align_vertical(), Align::Begin)
        // .set(fit_horizontal(), Fit::Children)
        // .set(fit_vertical(), Fit::Children)
    }
}

#[derive(Debug, Clone)]
pub struct Centered(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Centered);
impl ElementComponent for Centered {
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        Flow(self.0)
            .el()
            .set_default(orientation_vertical())
            .set_default(align_horizontal_center())
            .set_default(align_vertical_center())
            .set_default(fit_horizontal_none())
            .set_default(fit_vertical_none())
        // .set(orientation(), Orientation::Vertical)
        // .set(align_horizontal(), Align::Center)
        // .set(align_vertical(), Align::Center)
        // .set(fit_horizontal(), Fit::None)
        // .set(fit_vertical(), Fit::None)
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
    inner.on_spawned(move |_, id, _| set_id(Some(id)))
}
