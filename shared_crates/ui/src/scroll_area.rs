//! Defines a scroll area.
use ambient_element::{element_component, to_owned, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        app::window_scale_factor,
        ecs::children,
        input::{mouse_over, mouse_pickable_max, mouse_pickable_min},
        layout::{
            fit_horizontal_children, fit_horizontal_parent, fit_vertical_children, height, width,
        },
        rect::{background_color, border_radius},
        rendering::scissors_recursive,
        transform::{local_to_parent, local_to_world, translation},
    },
    messages,
};
use glam::{uvec4, vec3, vec4, Vec2, Vec3, Vec4};

use crate::{
    layout::{Flow, MeasureAbsolutePosition, MeasureSize},
    Rectangle, UIBase,
};

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
    let (ratio, _set_ratio) = hooks.use_state_with(|world| {
        let r = world.resource(window_scale_factor()).clone();
        r as f32
    });
    let (outer_size, set_outer_size) = hooks.use_state(Vec2::ZERO);
    let (inner_size, set_inner_size) = hooks.use_state(Vec2::ZERO);
    // this will be auto-calculated by inner height - outer height
    let scroll_height = if inner_size.y - outer_size.y > 0.0 {
        inner_size.y - outer_size.y
    } else {
        0.0
    };
    let mouse_over_count = hooks.use_ref_with(|_| 0);
    let bar_height = outer_size.y / (outer_size.y + scroll_height) * outer_size.y;
    let offset = scroll / scroll_height * (outer_size.y - bar_height);
    let id = hooks.use_ref_with(|_| None);
    let inner_flow_id = hooks.use_ref_with(|_| None);
    let (canvas_offset, set_canvas_offset) = hooks.use_state(Vec3::ZERO);

    hooks.use_frame({
        to_owned![id, mouse_over_count];
        move |world| {
            if let Some(id) = *id.lock() {
                let number = world.get(id, mouse_over()).unwrap_or(0);
                *mouse_over_count.lock() = number;
            }
        }
    });
    hooks.use_runtime_message::<messages::WindowMouseWheel>({
        to_owned![mouse_over_count];
        move |_world, event| {
            if *mouse_over_count.lock() == 0 {
                return;
            };
            let delta = event.delta;
            let mouse_pos = scroll + if event.pixels { delta.y } else { delta.y * 20. };
            set_scroll(mouse_pos.clamp(-scroll_height, 0.0));
        }
    });

    let canvas = MeasureSize::el(
        MeasureAbsolutePosition::el(UIBase::el(), set_canvas_offset),
        set_outer_size,
    )
    .on_spawned({
        to_owned![id];
        move |_world, canvas_id, _| {
            *id.lock() = Some(canvas_id);
        }
    })
    .init(mouse_pickable_min(), Vec3::ZERO)
    .init(mouse_pickable_max(), Vec3::ZERO)
    .init_default(children())
    .children(vec![
        // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
        MeasureSize::el(
            {
                let flow = Flow(vec![inner])
                    .el()
                    .with(scissors_recursive(), {
                        let (y, h) = if canvas_offset.y > 0.0 {
                            (
                                (canvas_offset.y * ratio) as u32,
                                (outer_size.y * ratio) as u32,
                            )
                        } else {
                            (0, ((outer_size.y + canvas_offset.y) * ratio) as u32)
                        };

                        let (x, w) = if canvas_offset.x > 0.0 {
                            (
                                (canvas_offset.x * ratio) as u32,
                                (outer_size.x * ratio) as u32,
                            )
                        } else {
                            (0, ((outer_size.x + canvas_offset.x) * ratio) as u32)
                        };
                        uvec4(x, y, w, h)
                    })
                    .on_spawned({
                        to_owned![inner_flow_id];
                        move |_world, flow_id, _| {
                            *inner_flow_id.lock() = Some(flow_id);
                        }
                    })
                    .with(translation(), vec3(0., scroll, 0.));
                match sizing {
                    ScrollAreaSizing::FitParentWidth => flow
                        .with_default(fit_vertical_children())
                        .with_default(fit_horizontal_parent())
                        .with(width(), outer_size.x),

                    ScrollAreaSizing::FitChildrenWidth => flow
                        .with_default(fit_vertical_children())
                        .with_default(fit_horizontal_children()),
                }
            },
            set_inner_size,
        ),
        if scroll_height > 0.0 {
            Rectangle::el()
                .with(width(), 5.)
                .with(height(), bar_height)
                .with(border_radius(), Vec4::ONE * 4.0)
                .with(background_color(), vec4(0.6, 0.6, 0.6, 1.0))
                .with_default(local_to_parent())
                .with_default(local_to_world())
                .with(translation(), vec3(outer_size.x - 5.0, -offset, -0.1))
        } else {
            Element::new()
        },
    ]);

    match sizing {
        ScrollAreaSizing::FitChildrenWidth => canvas.with(width(), inner_size.x),
        ScrollAreaSizing::FitParentWidth => canvas,
    }
}
