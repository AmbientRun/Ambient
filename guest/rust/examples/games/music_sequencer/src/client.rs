use ambient_api::{messages::Frame, prelude::*};

mod common;
use common::{hsv_to_rgb, BEAT_COUNT, SECONDS_PER_NOTE};

#[main]
pub fn main() {
    let mut cursor = 0;
    let mut last_note_time = time();
    let mut tree = Element::new().spawn_tree();
    Frame::subscribe(move |_| {
        let now = time();
        if now - last_note_time > SECONDS_PER_NOTE {
            cursor = (cursor + 1) % BEAT_COUNT;
            last_note_time = now;

            tree.migrate_root(&mut World, App::el(cursor));
        }
        tree.update(&mut World);
    });
}

#[element_component]
fn App(hooks: &mut Hooks, cursor: usize) -> Element {
    let mut tracks = hooks.use_query((components::track(), components::track_note_selection()));
    tracks.sort_by_key(|t| t.1 .0);

    FlowColumn::el(
        tracks
            .into_iter()
            .map(|(track_id, (_, track_selection))| Track::el(track_id, track_selection, cursor)),
    )
}

#[element_component]
fn Track(
    hooks: &mut Hooks,
    track_id: EntityId,
    track_selection: Vec<u32>,
    cursor: usize,
) -> Element {
    let track_name = entity::get_component(track_id, name()).unwrap_or_default();

    let (sound, _) = hooks.use_state_with(|_| {
        let url = entity::get_component(track_id, components::track_audio_url()).unwrap();
        audio::load(asset::url(url).unwrap())
    });

    let (last_cursor, set_last_cursor) = hooks.use_state(0);
    if cursor != last_cursor {
        if track_selection[cursor] != 0 {
            sound.play();
        }
        set_last_cursor(cursor);
    }

    FlowColumn::el([
        Text::el(track_name),
        FlowRow::el(
            track_selection
                .iter()
                .enumerate()
                .map(|(index, &selected_hue)| {
                    let is_on_cursor = cursor == index;
                    Button::new(Note::el(selected_hue, is_on_cursor), move |_| {
                        messages::Click::new(index as u32, track_id).send_server_reliable();
                    })
                    .style(ButtonStyle::Card)
                    .el()
                }),
        )
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_children())
        .with_default(fit_horizontal_children())
        .with_padding_even(10.)
        .with(space_between_items(), 2.),
    ])
}

#[element_component]
fn Note(_hooks: &mut Hooks, hue: u32, highlight: bool) -> Element {
    Rectangle
        .el()
        .with_background(match highlight {
            true => match hue == 0 {
                true => vec4(0.5, 0.5, 0.5, 1.),
                false => hsv_to_rgb(&[hue as f32, 0.7, 0.8]).extend(1.) * 2.2,
            },
            false => match hue == 0 {
                true => vec4(0.2, 0.2, 0.2, 1.),
                false => hsv_to_rgb(&[hue as f32, 0.7, 1.0]).extend(1.) * 2.2,
            },
        })
        .with(width(), 50.)
        .with(height(), 50.)
}
