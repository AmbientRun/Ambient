use ambient_api::{messages::Frame, prelude::*};
use components::note_selection;

mod common;
use common::{hsv_to_rgb, BEAT_COUNT, SECONDS_PER_NOTE, SOUNDS};

fn make_row(text: &str, note_selection_now: &[u32], cursor_now: usize, pos: usize) -> Element {
    let card_inner = |hue: u32, highlight: bool| {
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
    };
    FlowColumn::el([
        Text::el(text),
        FlowRow::el(
            note_selection_now[pos..pos + BEAT_COUNT]
                .iter()
                .enumerate()
                .map(|(index, &selected)| {
                    let is_on_cursor = cursor_now == index;
                    FlowRow::el([Button::new(card_inner(selected, is_on_cursor), move |_| {
                        messages::Click::new(u32::try_from(index + pos).unwrap())
                            .send_server_reliable();
                    })
                    .style(ButtonStyle::Card)
                    .el()])
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
fn App(_hooks: &mut Hooks, cursor: usize, notes: Vec<u32>) -> Element {
    FlowColumn::el(
        SOUNDS
            .iter()
            .enumerate()
            .map(|(i, (label, _))| make_row(label, &notes, cursor, i * BEAT_COUNT)),
    )
}

#[main]
pub fn main() {
    let sounds: Vec<_> = SOUNDS
        .iter()
        .map(|(_, path)| audio::load(asset::url(path).unwrap()))
        .collect();

    let mut cursor = 0;
    let mut last_note_time = time();
    let mut tree = Element::new().spawn_tree();
    Frame::subscribe(move |_| {
        let Some(notes) = entity::get_component(entity::synchronized_resources(), note_selection()) else { return; };

        let now = time();
        if now - last_note_time > SECONDS_PER_NOTE {
            cursor = (cursor + 1) % BEAT_COUNT;
            last_note_time = now;

            for (i, sound) in sounds.iter().enumerate() {
                if notes[i * BEAT_COUNT + cursor] != 0 {
                    sound.play();
                }
            }
            tree.migrate_root(&mut World, App::el(cursor, notes));
        }
        tree.update(&mut World);
    });
}
