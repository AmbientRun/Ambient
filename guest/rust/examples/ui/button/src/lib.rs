use ambient_api::{
    components::core::{app::ui_scene, game_objects::player_camera},
    concepts::make_orthographic_camera,
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{
    components::{
        camera::orthographic_from_window,
        player::{player, user_id},
    },
    ecs::World,
};
use ambient_ui_components::{text::Text, UIExt};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (count, set_count) = hooks.use_state(0);
    Text::el(format!("Hello world: {count}"))
        .with_clickarea()
        .on_mouse_down(move |_, _, _| set_count(count + 1))
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
                    .with(orthographic_from_window(), id)
                    .with_default(player_camera())
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
