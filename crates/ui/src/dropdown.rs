use ambient_core::transform::translation;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use closure::closure;
use glam::vec3;

use super::{FlowColumn, UIBase, UIExt};
use crate::{
    border_radius,
    layout::{margin, Borders},
    padding, tooltip_background_color, Corners, SMALL_ROUNDING, STREET,
};
use ambient_ui_components::UIExt2;

#[element_component]
pub fn Dropdown(_: &mut Hooks, content: Element, dropdown: Element, show: bool) -> Element {
    FlowColumn::el([
        content,
        if show {
            UIBase.el().children(vec![FlowColumn(vec![dropdown]).el().set(translation(), vec3(0., 0., -0.05))])
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
            .set(padding(), Borders::even(STREET))
            .with_background(tooltip_background_color().into())
            .set(border_radius(), Corners::even(SMALL_ROUNDING).into())
            .set(margin(), Borders::top(STREET)),
        show: hover,
    }
    .el()
    .with_clickarea()
    .on_mouse_enter(closure!(clone set_hover, |_, _| set_hover(true)))
    .on_mouse_leave(move |_, _| set_hover(false))
    .el()
}
