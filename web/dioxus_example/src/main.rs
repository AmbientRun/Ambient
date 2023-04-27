#![allow(non_snake_case)]
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{console, window};

fn main() {
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
