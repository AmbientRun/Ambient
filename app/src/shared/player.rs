use std::{io::Write, sync::Arc};

use ambient_audio::AudioListener;
use ambient_core::{
    camera::{active_camera, aspect_ratio_from_window},
    runtime,
};
use ambient_ecs::{query, query_mut, Entity, SystemGroup};
use ambient_element::{element_component, Element, Hooks};
use ambient_input::{
    event_focus_change, event_keyboard_input, event_mouse_input, event_mouse_motion, event_mouse_wheel, player_prev_raw_input,
    player_raw_input, ElementState, MouseScrollDelta, PlayerRawInput,
};
use ambient_network::{
    client::game_client,
    get_player_by_user_id,
    player::{local_user_id, player, user_id},
    DatagramHandlers,
};
use ambient_std::unwrap_log_err;
use ambient_world_audio::audio_listener;
use byteorder::{BigEndian, WriteBytesExt};
pub use components::game_objects::player_camera;
use glam::{Mat4, Vec3};
use parking_lot::Mutex;

const PLAYER_INPUT_DATAGRAM_ID: u32 = 5;

mod components {
    pub mod game_objects {
        use ambient_ecs::{components, Debuggable, Description, Name, Networked};

        components!("game_objects", {
            // attached to a camera entity to mark it as belonging to a player
            // player is located through user_id
            @[
                Networked, Debuggable,
                Name["Player Camera"],
                Description["If attached to a camera entity, this camera will be used as a player's primary camera.\nIf a `user_id` is specified, this camera will only be used for that player; otherwise, it will be used for every player.\nThis component is temporary and will likely be removed with the addition of clientside scripting."]
            ]
            player_camera: (),
        });
    }
}

pub fn init_all_components() {
    components::game_objects::init_components();
}

pub fn register_datagram_handler(handlers: &mut DatagramHandlers) {
    handlers.insert(
        PLAYER_INPUT_DATAGRAM_ID,
        Arc::new(|state, _assets, user_id, data| {
            let input: PlayerRawInput = unwrap_log_err!(bincode::deserialize(&data));
            let mut state = state.lock();
            if let Some(world) = state.get_player_world_mut(user_id) {
                if let Some(player_id) = get_player_by_user_id(world, user_id) {
                    world.set(player_id, player_raw_input(), input).ok();
                }
            }
        }),
    );
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "player/server_systems",
        vec![query(player()).spawned().to_system(|q, world, qs, _| {
            let player_ids = q.collect_ids(world, qs);
            for player_id in player_ids {
                world.add_components(player_id, Entity::new().with_default(player_raw_input()).with_default(player_prev_raw_input())).ok();
            }
        })],
    )
}

pub fn server_systems_final() -> SystemGroup {
    SystemGroup::new(
        "player/server_systems_final",
        vec![query_mut(player_prev_raw_input(), player_raw_input()).to_system(|q, world, qs, _| {
            for (_, prev, input) in q.iter(world, qs) {
                *prev = input.clone();
            }
        })],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "player/client_systems",
        vec![query(player_camera()).spawned().to_system(|q, world, qs, _| {
            // TEMP: This synchronises server cameras to the client. This is a temporary solution until this
            // is moved to/controlled by clientside WASM.

            let local = world.resource(local_user_id()).clone();
            for (id, _) in q.collect_cloned(world, qs) {
                let camera_user_id = world.get_ref(id, user_id());
                // Activate this camera if no user ID was specified or if this user ID matches
                // our local user ID
                if !camera_user_id.map_or(true, |uid| *uid == local) {
                    continue;
                }

                world
                    .add_components(
                        id,
                        Entity::new()
                            .with(active_camera(), 0.)
                            .with(audio_listener(), Arc::new(Mutex::new(AudioListener::new(Mat4::IDENTITY, Vec3::X * 0.2))))
                            .with(aspect_ratio_from_window(), ()),
                    )
                    .unwrap();
            }
        })],
    )
}

#[element_component]
pub fn PlayerRawInputHandler(hooks: &mut Hooks) -> Element {
    const PIXELS_PER_LINE: f32 = 5.0;

    let input = hooks.use_ref_with(|_| PlayerRawInput::default());
    let (has_focus, set_has_focus) = hooks.use_state(false);

    hooks.use_world_event({
        let input = input.clone();
        move |_world, event| {
            if let Some(event) = event.get_ref(event_keyboard_input()) {
                if let Some(keycode) = event.keycode {
                    let mut lock = input.lock();
                    match event.state {
                        ElementState::Pressed => {
                            lock.keys.insert(keycode);
                        }
                        ElementState::Released => {
                            lock.keys.remove(&keycode);
                        }
                    }
                }
            } else if let Some(event) = event.get_ref(event_mouse_input()) {
                let mut lock = input.lock();
                match event.state {
                    ElementState::Pressed => {
                        lock.mouse_buttons.insert(event.button);
                    }
                    ElementState::Released => {
                        lock.mouse_buttons.remove(&event.button);
                    }
                }
            } else if let Some(delta) = event.get(event_mouse_motion()) {
                input.lock().mouse_position += delta;
            } else if let Some(delta) = event.get(event_mouse_wheel()) {
                input.lock().mouse_wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * PIXELS_PER_LINE,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
            } else if let Some(focus) = event.get(event_focus_change()) {
                set_has_focus(focus);
            }
        }
    });
    hooks.use_frame(move |world| {
        if !has_focus {
            return;
        }

        if let Some(Some(gc)) = world.resource_opt(game_client()).cloned() {
            let runtime = world.resource(runtime()).clone();
            let input = input.clone();

            runtime.spawn(async move {
                let mut data = Vec::new();
                data.write_u32::<BigEndian>(PLAYER_INPUT_DATAGRAM_ID).unwrap();

                let msg = bincode::serialize(&*input.lock()).unwrap();
                data.write_all(&msg).unwrap();
                gc.connection.send_datagram(data.into()).ok();
            });
        }
    });

    Element::new()
}
