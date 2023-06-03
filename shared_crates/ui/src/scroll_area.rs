//! Defines a scroll area.
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    // app::{ui_scene, window_logical_size, window_physical_size},
    components::{
        ecs::children,
        rect::{background_color, border_radius},
        layout::{fit_horizontal_children, fit_horizontal_parent, layout_width_to_children, width, height},
        transform::translation,
    },
    messages,
};
use glam::{vec3, Vec2, vec4, Vec4};

use crate::{
    use_window_logical_resolution,
    layout::{Flow, MeasureSize},
    Rectangle,
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
    /// Fit to children width, height is screen and assign a max scroll down
    MaxScrollDown(f32),
}

/// A scroll area that can be used to scroll its child.
#[element_component]
pub fn ScrollArea(
    hooks: &mut Hooks,
    /// The scroll area sizing
    sizing: ScrollAreaSizing,
    /// The child element
    inner: Element
) -> Element {
    let (scroll, set_scroll) = hooks.use_state(0.);
    let (inner_size, set_inner_size) = hooks.use_state(Vec2::ZERO);
    let res = use_window_logical_resolution(hooks);
    let max_scroll_down = match sizing {
        ScrollAreaSizing::MaxScrollDown(v) => v,
        _ => 1080.0,
    };
    // TODO: This should be calculated from the size of the inner element
    let container_height = res.y as f32;
    let height_limit = -max_scroll_down;
    let bar_height = container_height * container_height / (container_height + max_scroll_down);
    let scroll_portion = -scroll / max_scroll_down;
    let offset = scroll_portion * (container_height - bar_height);
    hooks.use_runtime_message::<messages::WindowMouseWheel>({
        let set_scroll = set_scroll.clone();
        move |_world, event| {
        let delta = event.delta;
        let s = scroll + if event.pixels { delta.y } else { delta.y * 20. };
        set_scroll(s.clamp(height_limit, 0.0));
    }});

    match sizing {
        ScrollAreaSizing::FitChildrenWidth => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    MeasureSize::el(
                        Flow(vec![inner]).el()
                        .with_default(fit_horizontal_children())
                        .with(translation(), vec3(0., scroll, 0.)),
                        cb(move |size| {
                            set_inner_size(size);
                        }),
                    ),
                    Rectangle::el()
                    .with(width(), 10.)
                    .with(height(), bar_height)
                    .with(border_radius(), Vec4::ONE * 4.0)
                    .with(background_color(), vec4(0.2, 0.2, 0.2, 0.8))
                    .with(translation(), vec3(res.x as f32-10.0, offset, 0.))
                ])
                .with_default(layout_width_to_children())
                .with(width(), inner_size.x)
        }
        ScrollAreaSizing::FitParentWidth => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    Flow(vec![inner]).el().with_default(fit_horizontal_parent()).with(translation(), vec3(0., scroll, 0.)),
                ])
                .with_default(layout_width_to_children())
        }
        ScrollAreaSizing::MaxScrollDown(_) => {
            UIBase
                .el()
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    MeasureSize::el(
                        Flow(vec![inner]).el()
                            .with_default(fit_horizontal_children())
                            .with(translation(), vec3(0., scroll, 0.)),
                        cb(move |size| {
                            set_inner_size(size);
                        }),
                    ),
                    Rectangle::el()
                        .with(width(), 10.)
                        .with(height(), bar_height)
                        .with(border_radius(), Vec4::ONE * 4.0)
                        .with(background_color(), vec4(0.2, 0.2, 0.2, 0.8))
                        .with(translation(), vec3(res.x as f32-10.0, offset, 0.))
                ])
                .with_default(layout_width_to_children())
                .with(width(), inner_size.x)
        }
    }
}