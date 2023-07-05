//! Defines the [Node] element.

use crate::prelude::*;
use ambient_element::{element_component, to_owned, Element, ElementComponent, Hooks};
use ambient_guest_bridge::{
    components::{
        app::cursor_position,
        input::{mouse_over, mouse_pickable_max, mouse_pickable_min},
        layout::{
            align_vertical_center, fit_horizontal_parent, height, margin, min_height, padding,
            space_between_items, width,
        },
        rect::{background_color, border_color, border_radius, border_thickness},
        text::font_style,
        transform::{local_to_parent, local_to_world, translation},
    },
    messages,
};
use ambient_shared_types::{CursorIcon, ModifiersState, VirtualKeyCode};
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
use std::str::FromStr;

#[element_component]
/// A button UI element.
pub fn Node(hooks: &mut Hooks) -> Element {
    let in_id = hooks.use_ref_with(|_| None);
    let out_id = hooks.use_ref_with(|_| None);
    let mouse_over_count = hooks.use_ref_with(|_| 0);

    hooks.use_frame({
        to_owned![out_id, mouse_over_count];
        move |world| {
            if let Some(id) = *out_id.lock() {
                let next = world.get(id, mouse_over()).unwrap_or(0);
                let mut state = mouse_over_count.lock();
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
        }
    });

    hooks.use_runtime_message::<messages::WindowMouseInput>({
        to_owned![out_id, mouse_over_count];
        move |world, event| {
            if let Some(id) = *out_id.lock() {
                if *mouse_over_count.lock() > 0 {
                    if event.pressed && event.button == 0 {
                        println!("mouse left down");
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

#[element_component]
/// A button UI element.
pub fn Graph(hooks: &mut Hooks) -> Element {
    let (nodes, set_nodes) = hooks.use_state(vec![]);
    // let (nodes_pos, set_nodes_pos) = hooks.use_state(vec![]);
    hooks.use_runtime_message::<messages::WindowKeyboardInput>({
        to_owned![nodes];
        move |world, event| {
            // let modifiers = ModifiersState::from_bits(event.modifiers).unwrap();
            // if modifiers.contains(ModifiersState::SHIFT) {
            //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Crosshair);
            // } else {
            //     ambient_guest_bridge::window::set_cursor(world, CursorIcon::Default);
            // }

            if let Some(virtual_keycode) = event
                .keycode
                .as_deref()
                .and_then(|x| VirtualKeyCode::from_str(x).ok())
            {
                println!("key: {:?}", virtual_keycode);
                if virtual_keycode != VirtualKeyCode::I {
                    return;
                }
                if event.pressed {
                    let mut nodes = nodes.clone();
                    // let mut nodes_pos = nodes_pos.clone();
                    // nodes_pos.push(world.resource(cursor_position()).clone());
                    nodes.push(NodeInfo {
                        pos: world.resource(cursor_position()).clone(),
                    });
                    set_nodes(nodes);
                }
            }
            // if event.pressed && event.button == 0 {
            //     println!("mouse left down");
            //     println!("cursor pos: {:?}", world.resource(cursor_position()));
            //     let mut nodes = nodes.clone();
            //     nodes.push(NodeInfo {
            //         pos: world.resource(cursor_position()).clone(),
            //     });
            //     set_nodes(nodes);
            // }
        }
    });

    Group::el(
        nodes
            .iter()
            .enumerate()
            .map(move |(i, node)| Node::el())
            .collect::<Vec<_>>(),
    )
}

/// Node info.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct NodeInfo {
    /// pos
    pos: Vec2,
}

// #[derive(Debug, PartialEq, Clone, Copy)]
// pub enum NodeKind {
//     Number,
//     Bang,
//     Math,
// }
