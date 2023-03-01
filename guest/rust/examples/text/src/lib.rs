use ambient_api::{
    components::core::{
        app::{main_scene, ui_scene},
        camera::{orthographic_bottom, orthographic_left, orthographic_right, orthographic_top},
        game_objects::player_camera,
        transform::{lookat_center, translation},
    },
    concepts::{make_orthographic_camera, make_perspective_infinite_reverse_camera},
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::ecs::World;
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
    println!("{}", count);
    Text::el(format!("Hello world: {}", count))
}

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_orthographic_camera())
        .with(orthographic_left(), -300.)
        .with(orthographic_right(), 300.)
        .with(orthographic_top(), -300.)
        .with(orthographic_bottom(), 300.)
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
