//! Implements a dropdown element.

use ambient_element::{
    element_component, to_owned, use_state, Element, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::{
    layout::components::margin, rect::components::border_radius, transform::components::translation,
};
use glam::{vec3, Vec4};

use crate::{
    default_theme::{tooltip_background_color, SMALL_ROUNDING, STREET},
    layout::FlowColumn,
    UIBase, UIExt,
};

#[element_component]
/// A dropdown element: shows the `dropdown` when `show` is specified.
pub fn Dropdown(
    _: &mut Hooks,
    /// The content (always shown).
    content: Element,
    /// The dropdown to show.
    dropdown: Element,
    /// Whether or not to show the dropdown.
    show: bool,
) -> Element {
    FlowColumn::el([
        content,
        if show {
            UIBase.el().children(vec![FlowColumn(vec![dropdown])
                .el()
                .with(translation(), vec3(0., 0., -0.05))])
        } else {
            Element::new()
        },
    ])
}

#[element_component]
/// A tooltip element: shows the `tooltip` when the `inner` is hovered.
pub fn Tooltip(
    hooks: &mut Hooks,
    /// The element to render; when hovered over, `tooltip` will be shown.
    inner: Element,
    /// The tooltip to show.
    tooltip: Element,
) -> Element {
    let (hover, set_hover) = use_state(hooks, false);
    Dropdown {
        content: inner,
        dropdown: FlowColumn(vec![tooltip])
            .el()
            .with_padding_even(STREET)
            .with_background(tooltip_background_color().into())
            .with(border_radius(), Vec4::ONE * SMALL_ROUNDING)
            .with(margin(), Vec4::X * STREET),
        show: hover,
    }
    .el()
    .with_clickarea()
    .on_mouse_enter({
        to_owned![set_hover];
        move |_, _| set_hover(true)
    })
    .on_mouse_leave(move |_, _| set_hover(false))
    .el()
}
