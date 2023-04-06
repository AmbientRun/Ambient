use ambient_element::{element_component, to_owned, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::{layout::margin_top, rect::border_radius, transform::translation};
use glam::{vec3, Vec4};

use crate::{
    default_theme::{tooltip_background_color, SMALL_ROUNDING, STREET}, layout::FlowColumn, UIBase, UIExt
};

#[element_component]
pub fn Dropdown(_: &mut Hooks, content: Element, dropdown: Element, show: bool) -> Element {
    FlowColumn::el([
        content,
        if show {
            UIBase.el().children(vec![FlowColumn(vec![dropdown]).el().with(translation(), vec3(0., 0., -0.05))])
        } else {
            Element::new()
        },
    ])
}

#[element_component]
pub fn Tooltip(hooks: &mut Hooks, inner: Element, tooltip: Element) -> Element {
    let (hover, set_hover) = hooks.use_state(false);
    Dropdown {
        content: inner,
        dropdown: FlowColumn(vec![tooltip])
            .el()
            .with_padding_even(STREET)
            .with_background(tooltip_background_color().into())
            .with(border_radius(), Vec4::ONE * SMALL_ROUNDING)
            .with(margin_top(), STREET),
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
