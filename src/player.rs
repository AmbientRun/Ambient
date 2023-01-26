use std::sync::Arc;

use elements_audio::AudioListener;
use elements_core::{camera::active_camera, main_scene};
use elements_ecs::{components, query, SystemGroup};
use elements_network::player::{local_user_id, user_id};
use elements_world_audio::audio_listener;
use glam::{Mat4, Vec3};
use parking_lot::Mutex;

components!("game_objects", {
    player_camera: (),
});

pub fn client_systems() -> SystemGroup {
    SystemGroup::new(
        "player/client_systems",
        vec![query((player_camera(), user_id())).spawned().to_system(|q, world, qs, _| {
            // TEMP: This synchronises server cameras to the client. This is a temporary solution until this
            // is moved to/controlled by clientside scripting.

            let local = world.resource(local_user_id()).clone();
            for (id, (_, player_id)) in q.collect_cloned(world, qs) {
                if player_id == local {
                    world.add_component(id, active_camera(), 0.).unwrap();
                    world.add_component(id, main_scene(), ()).unwrap();

                    log::info!("Adding audio listener to: {id} {player_id:?}");
                    // Add a listener on the local camera for each client
                    world
                        .add_component(id, audio_listener(), Arc::new(Mutex::new(AudioListener::new(Mat4::IDENTITY, Vec3::X * 0.2))))
                        .unwrap();
                }
            }
        })],
    )
}
