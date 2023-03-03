use ambient_api::{
    components::core::{app::ui_scene, game_objects::player_camera},
    concepts::make_orthographic_camera,
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::{components::camera::orthographic_from_window, ecs::World};
use ambient_ui_components::text::Text;

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
    Text::el(format!("Hello world: {count}"))
}

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_orthographic_camera())
        .with_default(orthographic_from_window())
        .with_default(player_camera())
        .with_default(ui_scene())
        .spawn();

    let mut tree = App.el().spawn_tree();
    on(ambient_api::event::FRAME, move |_| {
        tree.update(&mut World);
        EventOk
    });

    EventOk
}
