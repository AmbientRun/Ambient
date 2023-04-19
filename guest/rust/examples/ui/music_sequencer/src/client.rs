use ambient_api::prelude::*;
use ambient_ui::prelude::*;
use components::{cursor, note_selection};

fn make_row(text: &str, note_selection_now: &[bool], cursor_now: u8, pos: u8) -> Element {
    let card_inner = |_selected: bool, highlight: bool| {
        FlowRow(vec![Text::el("")])
            .el()
            .with_background(match highlight {
                true => vec4(0.3, 0.3, 0.3, 1.),
                false => vec4(0.7, 0.7, 0.7, 1.),
            })
            .with_padding_even(20.)
    };
    FlowColumn::el(vec![
        Text::el(text),
        FlowRow::el(
            note_selection_now[pos as usize..pos as usize + 16]
                .iter()
                .enumerate()
                .map(|(index, &selected)| {
                    let is_on_cursor = cursor_now == index as u8;
                    FlowRow::el([Button::new(card_inner(selected, is_on_cursor), move |_| {
                        messages::Click::new(index as u8 + pos).send_server_reliable();
                    })
                    .toggled(selected)
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
fn App(hooks: &mut Hooks) -> Element {
    let items = hooks.use_query((note_selection(), cursor()));

    let (_id, (note_selection_now, cursor_now)) = &items[0];

    FlowColumn::el(vec![
        make_row("Kick Drum", note_selection_now, *cursor_now, 0),
        make_row("Snare Drum", note_selection_now, *cursor_now, 16),
        make_row("Closed Hihat", note_selection_now, *cursor_now, 32),
        make_row("Open Hihat", note_selection_now, *cursor_now, 48),
        make_row("Low Conga", note_selection_now, *cursor_now, 64),
        make_row("Mid Conga", note_selection_now, *cursor_now, 80),
        make_row("High Tom", note_selection_now, *cursor_now, 96),
        make_row("Mid Tom", note_selection_now, *cursor_now, 112),
    ])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
    let bd = audio::load(asset::url("assets/BD2500.ogg").unwrap());
    let sd = audio::load(asset::url("assets/SD7550.ogg").unwrap());
    let ch = audio::load(asset::url("assets/CH.ogg").unwrap());
    let oh = audio::load(asset::url("assets/OH75.ogg").unwrap());
    let lc = audio::load(asset::url("assets/LC00.ogg").unwrap());
    let mc = audio::load(asset::url("assets/MC00.ogg").unwrap());
    let ht = audio::load(asset::url("assets/HT75.ogg").unwrap());
    let mt = audio::load(asset::url("assets/MT75.ogg").unwrap());

    messages::Play::subscribe(move |_source, data| {
        let sounds = [&bd, &sd, &ch, &oh, &lc, &mc, &ht, &mt];
        if let Some(sound) = sounds.get(data.index as usize) {
            sound.play();
        }
    });
}
