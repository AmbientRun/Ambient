#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    ambient_web::init_ambient(true, true);
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    use_future(cx, (), |_| async move {
        ambient_web::start().await;
    });

    cx.render(rsx! {
        div {
            id: "my_comp",
        }
    })
}
