use ambient_api::prelude::*;
use common::NOTE_COUNT;
use components::note_selection;

mod common;

#[main]
pub async fn main() {
    entity::add_component(
        entity::synchronized_resources(),
        note_selection(),
        vec![false; NOTE_COUNT],
    );

    messages::Click::subscribe(move |_source, data| {
        entity::mutate_component(
            entity::synchronized_resources(),
            note_selection(),
            |selection| {
                selection[data.index as usize] = !selection[data.index as usize];
            },
        );
    });
}
