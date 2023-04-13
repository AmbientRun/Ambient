use ambient_api::prelude::*;

#[main]
pub fn main() {
    // TODO: load sound in client

    messages::Bonk::subscribe(|source, data| {
        // TODO: play sound here
        println!("[{source:?}] sent a msg => {:?}", data);
    });
}