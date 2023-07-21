use ambient_api::prelude::*;

#[main]
pub fn main() {
    // This module will receive messages from `server.rs`, and respond to them.
    messages::Local::subscribe(move |source, data| {
        println!("{source:?}: {data:?}");
        if let Some(id) = source.local() {
            messages::Local::new("Hi, back!").send_local(id);
        }
    });
}
