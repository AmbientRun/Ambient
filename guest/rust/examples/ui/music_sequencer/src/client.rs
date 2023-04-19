use ambient_api::prelude::*;
use ambient_ui_components::prelude::*;
use components::{cursor, note_selection};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let card_inner = |selected, highlight, _row| {
        FlowRow(vec![Text::el("")])
            .el()
            .with_background(match highlight {
                true => vec4(0.3, 0.3, 0.3, 1.),
                false => vec4(0.7, 0.7, 0.7, 1.),
            })
            .with_padding_even(20.)
    };
    let items = hooks.use_query((note_selection(), cursor()));

    let (_id, (_note_selection, _cursor)) = &items[0];

    FlowColumn::el(vec![
        FlowRow::el(
            _note_selection[0..16]
                .iter()
                .enumerate()
                .map(|(index, &selected)| {
                    let is_on_cursor = *_cursor == index as u8;
                    let row = 0;
                    FlowRow::el([
                        Button::new(card_inner(selected, is_on_cursor, row), move |_| {
                            messages::Click::new(index as u8).send_server_reliable();
                        })
                        .toggled(selected)
                        .style(ButtonStyle::Card)
                        .el(),
                    ])
                }),
        )
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_children())
        .with_default(fit_horizontal_children())
        .with_padding_even(10.)
        .with(space_between_items(), 2.),
        FlowRow::el(
            _note_selection[16..32]
                .iter()
                .enumerate()
                .map(|(index, &selected)| {
                    let is_on_cursor = *_cursor == index as u8;
                    let row = 1;
                    FlowRow::el([
                        Button::new(card_inner(selected, is_on_cursor, row), move |_| {
                            messages::Click::new(index as u8 + 16).send_server_reliable();
                        })
                        .toggled(selected)
                        .style(ButtonStyle::Card)
                        .el(),
                    ])
                }),
        )
        .with_background(vec4(0.1, 0.1, 0.1, 1.))
        .with_default(fit_vertical_children())
        .with_default(fit_horizontal_children())
        .with_padding_even(10.)
        .with(space_between_items(), 2.),
    ])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
    let mut bd = audio::load(asset::url("assets/BD2500.ogg").unwrap());
    let mut sd = audio::load(asset::url("assets/SD7550.ogg").unwrap());
    messages::Play::subscribe(move |_source, data| match data.index {
        0 => {
            bd.play();
        }
        1 => {
            sd.play();
        }
        _ => {}
    });
}
