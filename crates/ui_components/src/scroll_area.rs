use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_event_types::WINDOW_MOUSE_WHEEL;
use ambient_guest_bridge::components::{
    ecs::children,
    input::{event_mouse_wheel, event_mouse_wheel_pixels},
    layout::{fit_horizontal_parent, layout_width_to_children},
    transform::translation,
};
use glam::vec3;

use crate::{layout::Flow, UIBase};

#[derive(Debug, Clone)]
pub struct ScrollArea(pub Element);
impl ElementComponent for ScrollArea {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (scroll, set_scroll) = hooks.use_state(0.);
        hooks.use_event(WINDOW_MOUSE_WHEEL, move |_world, event| {
            if let Some(delta) = event.get(event_mouse_wheel()) {
                set_scroll(scroll + if event.get(event_mouse_wheel_pixels()).unwrap() { delta.y } else { delta.y * 20. });
            }
        });
        UIBase
            .el()
            .init_default(children())
            .children(vec![
                // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                Flow(vec![self.0]).el().set_default(fit_horizontal_parent()).set(translation(), vec3(0., scroll, 0.)),
            ])
            .set_default(layout_width_to_children())
    }
}
impl ScrollArea {
    pub fn el(element: Element) -> Element {
        Self(element).el()
    }
}
