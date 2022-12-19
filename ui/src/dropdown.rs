use std::sync::Arc;

use elements_core::transform::translation;
use elements_ecs::World;
use elements_element::{element_component, Element, ElementComponentExt, Hooks};
use elements_input::picking::{on_mouse_enter, on_mouse_leave};
use glam::vec3;

use super::{FlowColumn, UIBase, UIExt};
use crate::{
    border_radius, layout::{margin, Borders}, padding, tooltip_background_color, Corners, SMALL_ROUNDING, STREET
};

#[element_component]
pub fn Dropdown(_: &mut World, _: &mut Hooks, content: Element, dropdown: Element, show: bool) -> Element {
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
pub fn Tooltip(_: &mut World, hooks: &mut Hooks, inner: Element, tooltip: Element) -> Element {
    let (hover, set_hover) = hooks.use_state(false);
    Dropdown {
        content: inner,
        dropdown: FlowColumn(vec![tooltip])
            .el()
            .set(padding(), Borders::even(STREET))
            .with_background(tooltip_background_color())
            .set(border_radius(), Corners::even(SMALL_ROUNDING))
            .set(margin(), Borders::top(STREET)),
        show: hover,
    }
    .el()
    .with_clickarea()
    .listener(on_mouse_enter(), Arc::new(closure!(clone set_hover, |_, _| set_hover(true))))
    .listener(on_mouse_leave(), Arc::new(move |_, _| set_hover(false)))
}
