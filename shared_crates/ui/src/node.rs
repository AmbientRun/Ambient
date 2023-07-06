//! Defines the [Node] element.

use crate::prelude::*;
use ambient_element::{element_component, to_owned, Element, ElementComponent, Hooks};
use ambient_guest_bridge::{
    components::{
        app::cursor_position,
        ecs::parent,
        input::{mouse_over, mouse_pickable_max, mouse_pickable_min},
        layout::{
            align_vertical_center, fit_horizontal_parent, height, margin, min_height, padding,
            space_between_items, width,
        },
        rect::{
            background_color, border_color, border_radius, border_thickness, line_from, line_to,
            line_width,
        },
        text::font_style,
        transform::{local_to_parent, local_to_world, translation},
    },
    messages,
};
use ambient_shared_types::{CursorIcon, ModifiersState, VirtualKeyCode};
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};
use std::str::FromStr;

#[element_component]
/// A Graph Element that you can put lots of nodes
pub fn Graph(hooks: &mut Hooks) -> Element {
    let (nodes, set_nodes) = hooks.use_state(vec![]);
    let in_id = hooks.use_ref_with(|_| vec![]);
    let out_id = hooks.use_ref_with(|_| vec![]);
    let start_id = hooks.use_ref_with(|_| None);
    let (lines, set_lines) = hooks.use_state(vec![]);
    let (temp_line, set_temp_line) = hooks.use_state(None::<(Vec3, Vec3)>);
    let temp_line_toggle = hooks.use_ref_with(|_| false);
    hooks.use_frame({
        to_owned![lines, set_lines, temp_line, set_temp_line, temp_line_toggle];
        move |world| {
            if *temp_line_toggle.lock() {
                if let Some(line) = temp_line {
                    let mouse_pos = world.resource(cursor_position());
                    set_temp_line(Some((line.0, mouse_pos.extend(line.1.z))));
                }
            }

            for (index, ((from_id, from_pos), (to_id, to_pos))) in lines.iter().enumerate() {
                // let new_from_pos = world.get(*from_id, translation()).unwrap_or(Vec3::ZERO);
                // let new_to_pos = world.get(*to_id, translation()).unwrap_or(Vec3::ZERO);
                let ltw = world.get(*from_id, local_to_world()).unwrap();
                let (_, _, new_from_pos) = Mat4::to_scale_rotation_translation(&ltw);
                let new_from_pos = new_from_pos + vec3(5.0, 5.0, 0.0);

                if new_from_pos != *from_pos {
                    let mut l = lines.clone();
                    l[index] = ((*from_id, new_from_pos), (*to_id, *to_pos));
                    set_lines(l);
                }

                let ltw = world.get(*to_id, local_to_world()).unwrap();
                let (_, _, new_to_pos) = Mat4::to_scale_rotation_translation(&ltw);
                let new_to_pos = new_to_pos + vec3(5.0, 5.0, 0.0);

                if new_to_pos != *to_pos {
                    let mut l = lines.clone();
                    l[index] = ((*from_id, *from_pos), (*to_id, new_to_pos));
                    set_lines(l);
                }
            }
        }
    });

    hooks.use_runtime_message::<messages::WindowMouseInput>({
        to_owned![
            out_id,
            in_id,
            start_id,
            lines,
            set_lines,
            set_temp_line,
            temp_line_toggle
        ];
        move |world, event| {
            for id in out_id.lock().iter() {
                if event.pressed && event.button == 0 {
                    let mouse_over_state = world.get(*id, mouse_over()).unwrap_or(0);
                    if mouse_over_state > 0 {
                        let ltw = world.get(*id, local_to_world()).unwrap();
                        let (_, _, pos) = Mat4::to_scale_rotation_translation(&ltw);
                        let start_pos = pos + vec3(5.0, 5.0, 0.0);
                        *start_id.lock() = Some((id.clone(), start_pos));
                        *temp_line_toggle.lock() = true;
                        set_temp_line(Some((start_pos, start_pos)))
                    }
                }
            }

            for id in in_id.lock().iter() {
                if !event.pressed && event.button == 0 {
                    let mouse_over_state = world.get(*id, mouse_over()).unwrap_or(0);
                    if mouse_over_state > 0 {
                        if let Some(start_id) = *start_id.lock() {
                            let mut l = lines.clone();
                            let ltw = world.get(*id, local_to_world()).unwrap();
                            let (_, _, pos) = Mat4::to_scale_rotation_translation(&ltw);
                            let end_pos = pos + vec3(5.0, 5.0, 0.0);
                            l.push((start_id, (id.clone(), end_pos)));
                            set_lines(l);
                        } else {
                            println!("no start id");
                        }
                    } else {
                        println!("dropped to none inlet");
                    }
                }
            }
            if !event.pressed {
                *temp_line_toggle.lock() = false;
                set_temp_line(None);
            }
        }
    });

    hooks.use_runtime_message::<messages::WindowKeyboardInput>({
        to_owned![nodes];
        move |world, event| {
            if let Some(virtual_keycode) = event
                .keycode
                .as_deref()
                .and_then(|x| VirtualKeyCode::from_str(x).ok())
            {
                if virtual_keycode != VirtualKeyCode::I {
                    return;
                }
                if event.pressed {
                    let mut nodes = nodes.clone();
                    nodes.push(NodeInfo {
                        pos: world.resource(cursor_position()).clone().extend(-0.001),
                    });
                    set_nodes(nodes);
                }
            }
        }
    });
    let group_nodes = Group::el(
        nodes
            .iter()
            .map(move |node| {
                println!("node: {:?}", node);
                let inlet = Rectangle::el()
                    .with(width(), 10.)
                    .with(height(), 10.)
                    .with(background_color(), vec4(0.5, 0.8, 1.0, 1.0))
                    .with(border_radius(), Vec4::ONE * 5.)
                    .with(translation(), vec3(-5., 20., -0.1))
                    .init(mouse_pickable_min(), Vec3::ZERO)
                    .init(mouse_pickable_max(), Vec3::ZERO)
                    .on_spawned({
                        to_owned![in_id];
                        move |_, new_id, _| {
                            in_id.lock().push(new_id);
                        }
                    });
                let outlet = Rectangle::el()
                    .with(width(), 10.)
                    .with(height(), 10.)
                    .with(background_color(), vec4(0.5, 0.8, 1.0, 1.0))
                    .with(border_radius(), Vec4::ONE * 5.)
                    .with(translation(), vec3(195., 20., -0.1))
                    .init(mouse_pickable_min(), Vec3::ZERO)
                    .init(mouse_pickable_max(), Vec3::ZERO)
                    .on_spawned({
                        to_owned![out_id];
                        move |_, new_id, _| {
                            out_id.lock().push(new_id);
                        }
                    });
                let select = DropdownSelect {
                    content: Text::el("Select"),
                    on_select: cb(|_| {}),
                    items: vec![Text::el("First"), Text::el("Second")],
                    inline: false,
                }
                .el()
                .with_padding_even(10.);
                // .with_margin_even(10.);
                let body = Rectangle::el()
                    .with(background_color(), vec4(0.4, 0.4, 0.4, 0.6))
                    .with(width(), 200.)
                    .with(height(), 300.)
                    .with(translation(), vec3(0., 10., 0.))
                    .with(border_radius(), vec4(0., 0., 5., 5.))
                    .with(border_color(), vec4(0.2, 0.2, 0.2, 0.6))
                    .with(border_thickness(), 1.0)
                    .with_padding_even(10.)
                    .with_margin_even(10.)
                    .children(vec![select]);

                Rectangle::el()
                    .with(background_color(), vec4(0.2, 0.2, 0.2, 0.6))
                    .with(width(), 200.)
                    .with(height(), 10.)
                    .with(border_radius(), vec4(5., 5., 0., 0.))
                    .children(vec![inlet, outlet, body])
                    .with_dragarea()
                    .el()
                    .on_spawned({
                        to_owned![node];
                        move |world, new_id, _| {
                            println!("spawned node: {}", new_id);
                            world.set(new_id, translation(), node.pos).unwrap();
                        }
                    })

                // Node::el().on_spawned({
                //     to_owned![id_list, node];
                //     move |world, new_id, _| {
                //         println!("spawned node: {}", new_id);
                //         id_list.lock().push(new_id);
                //         // id_list.lock().push(new_id);
                //         world.set(new_id, translation(), node.pos).unwrap();
                //     }
                // })
            })
            .collect::<Vec<_>>(),
    );
    let group_edges = Group::el(
        lines
            .iter()
            .map(|((_, from), (_, to))| {
                // let from = world.get(l.0, translation()).unwrap_or(Vec3::ZERO);
                Line.el()
                    .with(line_from(), *from)
                    .with(line_to(), *to)
                    .with(line_width(), 2.)
                    .with(background_color(), vec4(0.5, 0.8, 1.0, 1.))
            })
            .collect::<Vec<_>>(),
    );
    let temp_line_el = {
        to_owned![temp_line_toggle, temp_line];
        if let Some(line) = temp_line {
            if *temp_line_toggle.lock() {
                Line.el()
                    .with(line_from(), line.0)
                    .with(line_to(), line.1)
                    .with(line_width(), 2.)
                    .with(background_color(), vec4(0.5, 0.8, 1.0, 1.))
            } else {
                Element::new()
            }
        } else {
            Element::new()
        }
    };
    Group::el([group_nodes, group_edges, temp_line_el])
}

#[element_component]
/// A Node UI element.
pub fn Node(hooks: &mut Hooks) -> Element {
    let in_id = hooks.use_ref_with(|_| None);
    let out_id = hooks.use_ref_with(|_| None);
    let in_mouse_over_count = hooks.use_ref_with(|_| 0);
    let out_mouse_over_count = hooks.use_ref_with(|_| 0);

    hooks.use_frame({
        to_owned![out_id, in_id, in_mouse_over_count, out_mouse_over_count];
        move |world| {
            if let Some(id) = *out_id.lock() {
                let next = world.get(id, mouse_over()).unwrap_or(0);
                let mut state = out_mouse_over_count.lock();
                // if *state == 0 && next > 0 {
                //     println!("mouse enter outlet");
                //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Arrow);
                // }
                // if *state > 0 && next == 0 {
                //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
                // }

                if next > 0 {
                    ambient_guest_bridge::window::set_cursor(world, CursorIcon::Grab);
                };

                //  else {
                //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
                // }

                *state = next;
            }
            if let Some(id) = *in_id.lock() {
                let next = world.get(id, mouse_over()).unwrap_or(0);
                let mut state = in_mouse_over_count.lock();
                // if *state == 0 && next > 0 {
                //     println!("mouse enter inlet");
                //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Arrow);
                // }
                // if *state > 0 && next == 0 {
                //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
                // }

                if next > 0 {
                    ambient_guest_bridge::window::set_cursor(world, CursorIcon::Grab);
                };

                //  else {
                //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
                // }

                *state = next;
            }
        }
    });

    hooks.use_runtime_message::<messages::WindowMouseInput>({
        to_owned![out_id, in_id, in_mouse_over_count, out_mouse_over_count];
        move |world, event| {
            if let Some(id) = *out_id.lock() {
                if *out_mouse_over_count.lock() > 0 {
                    if event.pressed && event.button == 0 {
                        println!("mouse left down at {:?}", id);
                        // set_moving(true);
                        // let mouse_pos = world.resource(cursor_position());
                        // let pos = world.get(id, translation()).unwrap_or(Vec3::ZERO);
                        // set_mouse_click_pos((mouse_pos.x, mouse_pos.y));
                        // set_click_pos((pos.x, pos.y));
                    } else {
                        // set_moving(false);
                    }
                }
            }
            if let Some(id) = *in_id.lock() {
                if *in_mouse_over_count.lock() > 0 {
                    if !event.pressed && event.button == 0 {
                        println!("mouse left up at {:?}", id);
                    }
                }
            }
        }
    });

    let inlet = Rectangle::el()
        .with(width(), 10.)
        .with(height(), 10.)
        .with(background_color(), vec4(0.5, 0.8, 1.0, 1.0))
        .with(border_radius(), Vec4::ONE * 5.)
        .with(translation(), vec3(-5., 20., -0.1))
        .init(mouse_pickable_min(), Vec3::ZERO)
        .init(mouse_pickable_max(), Vec3::ZERO)
        .on_spawned(move |_, new_id, _| {
            *in_id.lock() = Some(new_id);
        });
    let outlet = Rectangle::el()
        .with(width(), 10.)
        .with(height(), 10.)
        .with(background_color(), vec4(0.5, 0.8, 1.0, 1.0))
        .with(border_radius(), Vec4::ONE * 5.)
        .with(translation(), vec3(195., 20., -0.1))
        .init(mouse_pickable_min(), Vec3::ZERO)
        .init(mouse_pickable_max(), Vec3::ZERO)
        .on_spawned(move |_, new_id, _| {
            *out_id.lock() = Some(new_id);
        });
    let select = DropdownSelect {
        content: Text::el("Select"),
        on_select: cb(|_| {}),
        items: vec![Text::el("First"), Text::el("Second")],
        inline: false,
    }
    .el()
    .with_padding_even(10.)
    .with_margin_even(10.);
    let body = Rectangle::el()
        .with(background_color(), vec4(0.4, 0.4, 0.4, 0.6))
        .with(width(), 200.)
        .with(height(), 300.)
        .with(translation(), vec3(0., 10., 0.))
        .with(border_radius(), vec4(0., 0., 5., 5.))
        .with(border_color(), vec4(0.2, 0.2, 0.2, 0.6))
        .with(border_thickness(), 1.0)
        .with_padding_even(10.)
        .with_margin_even(10.)
        .children(vec![select]);

    Rectangle::el()
        .with(background_color(), vec4(0.2, 0.2, 0.2, 0.6))
        .with(width(), 200.)
        .with(height(), 10.)
        .with(border_radius(), vec4(5., 5., 0., 0.))
        .children(vec![inlet, outlet, body])
        .with_dragarea()
        .el()
}

/// Node info.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NodeInfo {
    /// pos
    pos: Vec3,
}
