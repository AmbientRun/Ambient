use ambient_api::{
    element::{use_state, use_state_with},
    prelude::*,
};
use ambient_friendly_id::friendly_id;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (screen, set_screen) = use_state(hooks, None);
    PageScreen::el([
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
    ])
}

#[element_component]
fn SubScreen(hooks: &mut Hooks, on_back: Cb<dyn Fn() + Sync + Send>) -> Element {
    let (screen, set_screen) = use_state(hooks, None);
    let (id, _) = use_state_with(hooks, |_| friendly_id());
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
