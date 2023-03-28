use ambient_api::prelude::*;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_ui_components::{default_theme::STREET, text::Text, UIExt};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (count, set_count) = hooks.use_state(0);
    hooks.use_spawn(move |_| {
        run_async(async move {
            let mut count = 0;
            loop {
                sleep(0.5).await;
                count += 1;
                set_count(count);
            }
        });
        Box::new(|_| {})
    });
    println!("{count}");
    Text::el(format!("We've counted to {count} now")).with_padding_even(STREET)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
