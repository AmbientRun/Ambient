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
        transform::translation,
    },
    messages,
};
use ambient_shared_types::CursorIcon;
use glam::{vec3, vec4, Vec3, Vec4};

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
        .with(width(), 5.)
        .with(height(), 10.)
        // .with(background_color(), vec4(0.6, 0.2, 0.6, 0.6))
        // .with(border_radius(), Vec4::ONE * 5.)
        .init(mouse_pickable_min(), Vec3::ZERO)
        .init(mouse_pickable_max(), Vec3::ZERO)
        .on_spawned(move |_, new_id, _| {
            *in_id.lock() = Some(new_id);
        });
    let outlet = Rectangle::el()
        .with(width(), 5.)
        .with(height(), 10.)
        // .with(background_color(), vec4(0.2, 0.2, 0.6, 0.6))
        // .with(border_radius(), Vec4::ONE * 5.)
        .with(translation(), vec3(60., 0., 0.))
        .init(mouse_pickable_min(), Vec3::ZERO)
        .init(mouse_pickable_max(), Vec3::ZERO)
        .on_spawned(move |_, new_id, _| {
            *out_id.lock() = Some(new_id);
        });
    Rectangle::el()
        .with(background_color(), vec4(0.2, 0.6, 0.6, 0.6))
        .with(width(), 60.)
        .with(height(), 30.)
        .children(vec![inlet, outlet])
        .with_dragarea()
        .el()
}

#[element_component]
/// A button UI element.
pub fn Graph(hooks: &mut Hooks) -> Element {
    crate::text::Text::el("Graph")
}
