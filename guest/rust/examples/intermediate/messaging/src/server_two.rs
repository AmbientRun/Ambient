use ambient_api::prelude::*;
use packages::this::messages::{HelloWithoutBody, Local};

#[main]
pub fn main() {
    // This module will receive messages from `server.rs`, and respond to them.
    Local::subscribe(move |ctx, data| {
        println!("{ctx:?}: {data:?}");
        if let Some(id) = ctx.local() {
            Local::new("Hi, back!").send_local(id);
        }
    });

    HelloWithoutBody::subscribe(move |ctx, _| {
        println!("HelloWithoutBody: {ctx:?}");
    });
}
