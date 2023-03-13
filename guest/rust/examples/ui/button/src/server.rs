use ambient_api::{
    components::core::app::ui_scene, concepts::make_orthographic_camera, prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        input::event_mouse_input,
        player::{player, user_id},
        transform::translation,
    },
    ecs::World,
};
use ambient_ui_components::{text::Text, UIExt};

pub async fn main() -> EventResult {
    spawn_query((player(), user_id())).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(make_orthographic_camera())
                    .with(orthographic_from_window(), id)
                    .with_default(ui_scene()),
            );
        }
    });

    let mut tree = App.el().spawn_tree();
    on(ambient_api::event::FRAME, move |_| {
        tree.update(&mut World);
        EventOk
    });

    on(ambient_api::event::WORLD_EVENT, move |data| {
        if let Some(event) = data.get(event_mouse_input()) {
            println!("mouse: {:?}", event);
        }
        EventOk
    });

    EventOk
}

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (mouse_is_over, set_mouse_is_over) = hooks.use_state(false);

    Text::el(if mouse_is_over {
        "MOUSE IS OVER"
    } else {
        "Move the mouse here"
    })
    .with_clickarea()
    .on_mouse_enter({
        let set_mouse_is_over = set_mouse_is_over.clone();
        move |_, _| set_mouse_is_over(true)
    })
    .on_mouse_leave(move |_, _| set_mouse_is_over(false))
    .el()
    .set(translation(), vec3(100., 100., 0.))
}
