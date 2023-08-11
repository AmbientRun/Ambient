use ambient_api::{
    core::{
        app::components::name,
        player::components::{player, user_id},
    },
    entity::{resources, synchronized_resources},
    prelude::*,
};
use embers::ambient_example_music_sequencer::{
    assets,
    components::{bpm, next_player_hue, player_hue, track, track_audio_url, track_note_selection},
    messages::{Click, SetBpm},
};

mod common;

#[main]
pub async fn main() {
    entity::add_component(resources(), next_player_hue(), 0);
    entity::add_component(synchronized_resources(), bpm(), 120);

    // Create the tracks.
    for (idx, (track_name, track_url)) in [
        ("Kick Drum", "assets/BD2500.wav"),
        ("Snare Drum", "assets/SD7550.wav"),
        ("Closed Hihat", "assets/CH.wav"),
        ("Open Hihat", "assets/OH75.wav"),
        ("Low Conga", "assets/LC00.wav"),
        ("Mid Conga", "assets/MC00.wav"),
        ("High Tom", "assets/HT75.wav"),
        ("Mid Tom", "assets/MT75.wav"),
    ]
    .iter()
    .enumerate()
    {
        Entity::new()
            .with(name(), track_name.to_string())
            .with(track(), idx as u32)
            .with(track_audio_url(), assets::url(track_url))
            .with(track_note_selection(), vec![0; common::NOTE_COUNT])
            .spawn();
    }

    // When a player spawns, give them a color.
    spawn_query(user_id())
        .requires(player())
        .bind(move |players| {
            for (player, _player_user_id) in players {
                let mut h = entity::get_component(resources(), next_player_hue()).unwrap();
                h = (h + 103) % 360;
                entity::add_component(player, player_hue(), h);
                entity::set_component(resources(), next_player_hue(), h);
            }
        });

    // When a player requests a BPM change, update it.
    SetBpm::subscribe(|_source, data| {
        entity::set_component(synchronized_resources(), bpm(), data.bpm);
    });

    // When a player clicks on a note, toggle it.
    Click::subscribe(move |source, data| {
        let id = source.client_entity_id().unwrap();
        let color_to_set = entity::get_component(id, player_hue()).unwrap();

        entity::mutate_component(data.track_id, track_note_selection(), |selection| {
            if selection[data.index as usize] == color_to_set {
                selection[data.index as usize] = 0;
            } else {
                selection[data.index as usize] = color_to_set;
            }
        });
    });
}
