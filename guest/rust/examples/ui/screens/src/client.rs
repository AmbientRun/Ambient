use ambient_api::prelude::*;
use ambient_cb::{cb, Cb};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_friendly_id::friendly_id;
use ambient_ui_components::{
    button::Button,
    screens::{PageScreen, ScreenContainer},
    text::Text,
    FocusRoot,
};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (screen, set_screen) = hooks.use_state(None);
    FocusRoot::el([PageScreen::el([
        ScreenContainer(screen).el(),
        Text::el("RootScreen"),
        Button::new("Open sub screen", move |_| {
            set_screen(Some(SubScreen::el(cb({
                let set_screen = set_screen.clone();
                move || {
                    set_screen(None);
                }
            }))))
        })
        .el(),
    ])])
}

#[element_component]
fn SubScreen(hooks: &mut Hooks, on_back: Cb<dyn Fn() + Sync + Send>) -> Element {
    let (screen, set_screen) = hooks.use_state(None);
    let (id, _) = hooks.use_state_with(|_| friendly_id());
    PageScreen::el([
        ScreenContainer(screen).el(),
        Text::el(format!("SubScreen {id}")),
        Button::new("Back", move |_| on_back()).el(),
        Button::new("Open sub screen", {
            let set_screen = set_screen.clone();
            move |_| {
                set_screen(Some(SubScreen::el(cb({
                    let set_screen = set_screen.clone();
                    move || {
                        set_screen(None);
                    }
                }))))
            }
        })
        .el(),
    ])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
