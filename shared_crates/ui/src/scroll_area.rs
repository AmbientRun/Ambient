//! Defines a scroll area.
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        ecs::children,
        layout::{fit_horizontal_children, fit_horizontal_parent, layout_width_to_children, width},
        transform::translation,
    },
    messages,
};
use glam::{vec3, Vec2};

use crate::{
    layout::{Flow, MeasureSize},
    UIBase,
};
use ambient_cb::cb;

/// Sizing config of a scroll area
#[derive(Debug, Clone)]
pub enum ScrollAreaSizing {
    /// Resizes the scroll area to fit the width of its children
    FitChildrenWidth,
    /// Assumes the width from the parent and propagates it to the children
    FitParentWidth,
}

/// A scroll area that can be used to scroll its child.
#[element_component]
pub fn ScrollArea(
    hooks: &mut Hooks,
    /// The scroll area sizing
    sizing: ScrollAreaSizing,
    /// The child element
    inner: Element,
) -> Element {
    let (scroll, set_scroll) = hooks.use_state(0.);
    let (inner_size, set_inner_size) = hooks.use_state(Vec2::ZERO);
    hooks.use_runtime_message::<messages::WindowMouseWheel>(move |_world, event| {
        let delta = event.delta;
        set_scroll(scroll + if event.pixels { delta.y } else { delta.y * 20. });
    });
    match sizing {
        ScrollAreaSizing::FitChildrenWidth => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    MeasureSize::el(
                        Flow(vec![inner])
                            .el()
                            .with_default(fit_horizontal_children())
                            .with(translation(), vec3(0., scroll, 0.)),
                        cb(move |size| {
                            set_inner_size(size);
                        }),
                    ),
                ])
                .with(width(), inner_size.x)
        }
        ScrollAreaSizing::FitParentWidth => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    Flow(vec![inner])
                        .el()
                        .with_default(fit_horizontal_parent())
                        .with(translation(), vec3(0., scroll, 0.)),
                ])
                .with_default(layout_width_to_children())
        }
    }
}
