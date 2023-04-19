use ambient_api::prelude::*;
use components::{note_selection, cursor};

#[main]
pub async fn main() {
    let mut value = 0_u8;
    let mother = Entity::new()
    .with(note_selection(), vec![false; 128])
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
        let v = entity::get_component(mother, note_selection()).unwrap();
        for i in 0..8 {
            let index = value as usize + i * 16;
            if v[index] {
                messages::Play::new(i as u8).send_client_broadcast_reliable();
            }
        }
        sleep(0.125).await;
    }
}