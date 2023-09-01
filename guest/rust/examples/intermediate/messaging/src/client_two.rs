use ambient_api::prelude::*;
use packages::this::messages::Local;

#[main]
pub fn main() {
    // This module will receive messages from `client.rs`, and respond to them.
    Local::subscribe(move |ctx, data| {
        println!("{ctx:?}: {data:?}");
        if let Some(id) = ctx.local() {
            Local::new("Hi, back!").send_local(id);
        }
    });
}
