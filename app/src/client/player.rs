use std::str::FromStr;

use ambient_core::window::cursor_position;
use ambient_ecs::{query_mut, SystemGroup, World};
use ambient_element::{element_component, Element, Hooks};
use ambient_input::{
    event_focus_change, event_keyboard_input, event_mouse_input, event_mouse_wheel, event_mouse_wheel_pixels, keycode, mouse_button,
    player_prev_raw_input, player_raw_input, PlayerRawInput,
};
use ambient_network::client::game_client;
use ambient_shared_types::events::{WINDOW_FOCUSED, WINDOW_KEYBOARD_INPUT, WINDOW_MOUSE_INPUT, WINDOW_MOUSE_MOTION, WINDOW_MOUSE_WHEEL};
use ambient_window_types::VirtualKeyCode;

pub fn systems_final() -> SystemGroup {
    SystemGroup::new(
        "player/client_systems_final",
        vec![query_mut(player_prev_raw_input(), player_raw_input()).to_system(|q, world, qs, _| {
            for (_, prev, input) in q.iter(world, qs) {
                *prev = input.clone();
            }
        })],
    )
}

#[element_component]
pub fn PlayerRawInputHandler(hooks: &mut Hooks) -> Element {
    const PIXELS_PER_LINE: f32 = 5.0;

    // Assume window has focus
    let (has_focus, set_has_focus) = hooks.use_state(true);
    hooks.use_event(WINDOW_FOCUSED, move |_, event| {
        if let Some(focus) = event.get(event_focus_change()) {
            set_has_focus(focus);
        }
    });

    fn process_input(ui_world: &World, has_focus: bool, processor: impl Fn(&mut PlayerRawInput)) {
        if !has_focus {
            return;
        }

        let Some(Some(gc)) = ui_world.resource_opt(game_client()).cloned() else {
            return;
        };
        gc.with_physics_world(|w| {
            let input = w.resource_mut(player_raw_input());
            processor(input);
        });
    }

    hooks.use_event(WINDOW_KEYBOARD_INPUT, move |world, event| {
        process_input(world, has_focus, |input| {
            if let Some(pressed) = event.get(event_keyboard_input()) {
                if let Some(keycode) = event.get_ref(keycode()) {
                    let keycode = VirtualKeyCode::from_str(keycode).unwrap();
                    if pressed {
                        input.keys.insert(keycode);
                    } else {
                        input.keys.remove(&keycode);
                    }
                }
            }
        });
    });

    hooks.use_event(WINDOW_MOUSE_INPUT, move |world, event| {
        process_input(world, has_focus, |input| {
            if let Some(pressed) = event.get(event_mouse_input()) {
                if pressed {
                    input.mouse_buttons.insert(event.get(mouse_button()).unwrap().into());
                } else {
                    input.mouse_buttons.remove(&event.get(mouse_button()).unwrap().into());
                }
            }
        });
    });

    hooks.use_event(WINDOW_MOUSE_MOTION, move |world, _| {
        process_input(world, has_focus, |input| {
            input.mouse_position = *world.resource(cursor_position());
        });
    });

    hooks.use_event(WINDOW_MOUSE_WHEEL, move |world, event| {
        process_input(world, has_focus, |input| {
            if let Some(delta) = event.get(event_mouse_wheel()) {
                input.mouse_wheel += match event.get(event_mouse_wheel_pixels()).unwrap() {
                    false => delta.y * PIXELS_PER_LINE,
                    true => delta.y,
                };
            }
        });
    });

    Element::new()
}
