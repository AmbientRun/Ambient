use ambient_api::{
    components::core::app::ui_scene, concepts::make_orthographic_camera, prelude::*,
};
use ambient_cb::{cb, Cb};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_friendly_id::friendly_id;
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        player::{player, user_id},
    },
    ecs::World,
};
use ambient_ui_components::{
    button::Button,
    screens::{PageScreen, ScreenContainer},
    text::Text,
    FocusRoot,
};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (screen, set_screen) = hooks.use_state(None);
    FocusRoot(vec![PageScreen(vec![
        ScreenContainer(screen).el(),
        Text::el("RootScreen"),
        Button::new("Open sub screen", move |_| {
            set_screen(Some(
                SubScreen {
                    on_back: cb({
                        let set_screen = set_screen.clone();
                        move || {
                            set_screen(None);
                        }
                    }),
                }
                .el(),
            ))
        })
        .el(),
    ])
    .el()])
    .el()
}

#[element_component]
fn SubScreen(hooks: &mut Hooks, on_back: Cb<dyn Fn() + Sync + Send>) -> Element {
    let (screen, set_screen) = hooks.use_state(None);
    let (id, _) = hooks.use_state_with(|_| friendly_id());
    PageScreen(vec![
        ScreenContainer(screen).el(),
        Text::el(format!("SubScreen {id}")),
        Button::new("Back", move |_| on_back()).el(),
        Button::new("Open sub screen", {
            let set_screen = set_screen.clone();
            move |_| {
                set_screen(Some(
                    SubScreen {
                        on_back: cb({
                            let set_screen = set_screen.clone();
                            move || {
                                set_screen(None);
                            }
                        }),
                    }
                    .el(),
                ))
            }
        })
        .el(),
        // Button::new("Prompt", {
        //     let set_screen = set_screen.clone();
        //     move |_| {
        //         set_screen(Some(
        //             Prompt::new("Testy", set_screen.clone(), |_, _| {}).el(),
        //         ));
        //     }
        // })
        // .el(),
        // Button::new("Editor Prompt", move |_| {
        //     set_screen(Some(
        //         EditorPrompt::new(
        //             "Testy",
        //             "Something".to_string(),
        //             set_screen.clone(),
        //             |_, _| {},
        //         )
        //         .el(),
        //     ));
        // })
        // .el(),
    ])
    .el()
}

#[main]
pub async fn main() -> EventResult {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_orthographic_camera())
                    .with(orthographic_from_window(), EntityId::resources())
                    .with_default(ui_scene()),
            );
        }
    });

    let mut tree = App.el().spawn_tree();
    on(ambient_api::event::FRAME, move |_| {
        tree.update(&mut World);
        EventOk
    });

    EventOk
}
