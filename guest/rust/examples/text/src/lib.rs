use ambient_api::{
    components::core::{
        app::main_scene,
        game_objects::player_camera,
        transform::{lookat_center, translation},
    },
    concepts::make_perspective_infinite_reverse_camera,
    prelude::*,
};
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_ui_components::text::Text;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    Text::el("Hello world").set_default(main_scene())
}

#[main]
pub async fn main() -> EventResult {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with_default(player_camera())
        .with(translation(), vec3(5., 5., 4.))
        .with(lookat_center(), vec3(0., 0., 0.))
        .spawn();

    let tree = App.el().spawn_tree();

    EventOk
}
