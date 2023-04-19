use ambient_api::prelude::*;
use components::{note_selection, cursor};

#[main]
pub async fn main() {
    let mut value = 0_u8;
    let mother = Entity::new()
    .with(note_selection(), vec![false; 32])
    .with(cursor(), 0)
    .spawn();

    messages::Click::subscribe(move |_source, data| {
        let mut v = entity::get_component(mother, note_selection()).unwrap();
        v[data.index as usize] = !v[data.index as usize];
        entity::set_component(mother, note_selection(), v);
    });
    loop {
        value = (value + 1) % 16;
        entity::set_component(mother, cursor(), value);
        let mut v = entity::get_component(mother, note_selection()).unwrap();
        if v[value as usize] {
            messages::Play::new(0_u8).send_client_broadcast_reliable();
        };
        if v[value as usize + 16] {
            messages::Play::new(1_u8).send_client_broadcast_reliable();
        };
        sleep(0.125).await;
    }
}