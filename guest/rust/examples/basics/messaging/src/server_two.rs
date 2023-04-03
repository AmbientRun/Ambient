use ambient_api::prelude::*;

#[main]
pub fn main() {
    messages::Local::subscribe(move |source, data| {
        println!("{source:?}: {data:?}");
        if let Some(id) = source.local() {
            messages::Local::new("Hi, back!").send(Target::Local(id));
        }
    });
}
