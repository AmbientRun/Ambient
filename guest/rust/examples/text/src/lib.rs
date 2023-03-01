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
use ambient_ui_components::text::Text;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    Text::el("Hello world")
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

    let tree = App.el().spawn_tree();

    EventOk
}
