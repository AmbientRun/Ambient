//! Defines a scroll area.
use ambient_element::{element_component, to_owned, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        app::window_scale_factor,
        ecs::{children, parent},
        input::{mouse_over, mouse_pickable_max, mouse_pickable_min},
        layout::{
            fit_horizontal_children, fit_horizontal_parent, fit_vertical_children, height,
            layout_width_to_children, width,
        },
        rect::{background_color, border_radius},
        rendering::scissors_recursive,
        transform::{local_to_parent, local_to_world, translation},
    },
    messages,
};
use glam::{uvec4, vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};

use crate::{
    layout::{Flow, MeasureSize},
    Rectangle, UIBase,
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
    let (ratio, _set_ratio) = hooks.use_state_with(|world| {
        let r = world.resource(window_scale_factor()).clone();
        r as f32
    });
    let (min_width, set_min_width) = hooks.use_state(200.);
    let (min_height, set_min_height) = hooks.use_state(200.);
    let (scroll_height, set_scroll_height) = hooks.use_state(500.);

    let mouse_over_count = hooks.use_ref_with(|_| 0);
    let bar_height = min_height / (min_height + scroll_height) * min_height;
    let offset = scroll / scroll_height * (min_height - bar_height);
    let id = hooks.use_ref_with(|_| None);
    let inner_flow_id = hooks.use_ref_with(|_| None);
    let (canvas_offset, set_canvas_offset) = hooks.use_state(Vec2::ZERO);

    hooks.use_frame({
        to_owned![id, mouse_over_count, set_min_width];
        move |world| {
            if let Some(id) = *id.lock() {
                let number = world.get(id, mouse_over()).unwrap_or(0);
                // println!("scrollarea mouse over {} count: {:?}", id, number);
                *mouse_over_count.lock() = number;
                let canvas_local_to_world = world.get(id, local_to_world()).unwrap();
                let (_, _, pos_world) = Mat4::to_scale_rotation_translation(&canvas_local_to_world);
                set_canvas_offset(vec2(pos_world.x, pos_world.y));

                let p = world.get(id, parent());
                if let Ok(parent_id) = p {
                    let w = world.get(parent_id, width());
                    let h = world.get(parent_id, height());
                    if let Ok(w) = w {
                        set_min_width(w);
                    } else {
                        println!("no width");
                    }
                    if let Ok(h) = h {
                        set_min_height(h);
                    } else {
                        println!("no height");
                    }
                }
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
            // println!("scroll: {:?}", scroll);
        }
    });
    match sizing {
        ScrollAreaSizing::FitChildrenWidth => {
            let canvas = UIBase::el()
                // Rectangle::el()
                // .with(background_color(), vec4(0.0, 0.3, 0.0, 0.6))
                .init_default(children())
                .init(mouse_pickable_min(), Vec3::ZERO)
                .init(mouse_pickable_max(), Vec3::ZERO)
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly
                    // so had to introduce a Flow in between
                    MeasureSize::el(
                        Flow(vec![inner])
                            .el()
                            .with_default(fit_vertical_children())
                            .with_default(fit_horizontal_children())
                            .with(scissors_recursive(), {
                                let (y, h) = if canvas_offset.y > 0.0 {
                                    (
                                        (canvas_offset.y * ratio) as u32,
                                        (min_height * ratio) as u32,
                                    )
                                } else {
                                    let h_check = ((min_height + canvas_offset.y) * ratio) as u32;
                                    // h <= 0.0 will panic
                                    if h_check != 0 {
                                        (0, h_check)
                                    } else {
                                        (0, 1)
                                    }
                                };

                                let (x, w) = if canvas_offset.x > 0.0 {
                                    ((canvas_offset.x * ratio) as u32, (min_width * ratio) as u32)
                                } else {
                                    let w_check = ((min_width + canvas_offset.x) * ratio) as u32;
                                    // w <= 0.0 will panic
                                    if w_check != 0 {
                                        (0, w_check)
                                    } else {
                                        (0, 1)
                                    }
                                };
                                // println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);
                                uvec4(x, y, w, h)
                            })
                            .with(translation(), vec3(0., scroll, 0.)),
                        cb(move |size| {
                            println!("messured inner size: {:?}", size);
                            if size.y - min_height > 0.0 {
                                set_scroll_height(size.y - min_height);
                            } else {
                                set_scroll_height(0.0);
                            }
                        }),
                    ),
                    if scroll_height > 0.0 {
                        Rectangle::el()
                            .with(width(), 5.)
                            .with(height(), bar_height)
                            .with(border_radius(), Vec4::ONE * 4.0)
                            .with(background_color(), vec4(0.6, 0.6, 0.6, 1.0))
                            .with_default(local_to_parent())
                            .with_default(local_to_world())
                            .with(translation(), vec3(min_width - 5.0, -offset, 0.1))
                    } else {
                        UIBase.el()
                    },
                ])
                .on_spawned({
                    to_owned![id];
                    move |_world, canvas_id, _| {
                        *id.lock() = Some(canvas_id);
                    }
                })
                .with(height(), min_height)
                .with(width(), min_width);
            canvas
        }
        ScrollAreaSizing::FitParentWidth => {
            let canvas = UIBase::el()
                // .with(background_color(), vec4(0.3, 0.0, 0.0, 0.6))
                .on_spawned({
                    to_owned![id];
                    move |_world, canvas_id, _| {
                        *id.lock() = Some(canvas_id);
                    }
                })
                .with(height(), min_height)
                .with(width(), min_width)
                .init(mouse_pickable_min(), Vec3::ZERO)
                .init(mouse_pickable_max(), Vec3::ZERO)
                .init_default(children())
                .children(vec![
                    // TODO: For some reason it didn't work to set the translation on self.0 directly, so had to introduce a Flow in between
                    MeasureSize::el(
                        Flow(vec![inner])
                            .el()
                            .with_default(fit_vertical_children())
                            .with_default(fit_horizontal_parent())
                            .with(scissors_recursive(), {
                                let (y, h) = if canvas_offset.y > 0.0 {
                                    (
                                        (canvas_offset.y * ratio) as u32,
                                        (min_height * ratio) as u32,
                                    )
                                } else {
                                    let h_check = ((min_height + canvas_offset.y) * ratio) as u32;
                                    // h <= 0.0 will panic
                                    if h_check != 0 {
                                        (0, h_check)
                                    } else {
                                        (0, 1)
                                    }
                                };

                                let (x, w) = if canvas_offset.x > 0.0 {
                                    ((canvas_offset.x * ratio) as u32, (min_width * ratio) as u32)
                                } else {
                                    let w_check = ((min_width + canvas_offset.x) * ratio) as u32;
                                    // w <= 0.0 will panic
                                    if w_check != 0 {
                                        (0, w_check)
                                    } else {
                                        (0, 1)
                                    }
                                };
                                // println!("x: {}, y: {}, w: {}, h: {}", x, y, w, h);
                                uvec4(x, y, w, h)
                            })
                            .on_spawned({
                                to_owned![inner_flow_id];
                                move |_world, flow_id, _| {
                                    *inner_flow_id.lock() = Some(flow_id);
                                }
                            })
                            .with(translation(), vec3(0., scroll, 0.)),
                        cb(move |size| {
                            if size.y - min_height > 0.0 {
                                set_scroll_height(size.y - min_height);
                            } else {
                                set_scroll_height(0.0);
                            }
                        }),
                    ),
                    if scroll_height > 0.0 {
                        Rectangle::el()
                            .with(width(), 5.)
                            .with(height(), bar_height)
                            .with(border_radius(), Vec4::ONE * 4.0)
                            .with(background_color(), vec4(0.6, 0.6, 0.6, 1.0))
                            .with_default(local_to_parent())
                            .with_default(local_to_world())
                            .with(translation(), vec3(min_width - 5.0, -offset, 0.1))
                    } else {
                        UIBase.el()
                    },
                ])
                .with_default(layout_width_to_children());
            canvas
        }
    }
}
