use ambient_api::{core::transform::components::translation, prelude::*};
use ambient_brand_theme::AmbientInternalStyle;
use packages::this::components::player_count;

#[main]
pub fn main() {
    LivePlayerCountUI::el().spawn_interactive();
}

// DISPLAYS TEMPERATURE of player
#[element_component]
pub fn LivePlayerCountUI(hooks: &mut Hooks) -> Element {
    let player_count =
        ambient_api::element::use_entity_component(hooks, packages::this::entity(), player_count());

    Flow::el(if let Some(player_count) = player_count {
        [Text::el(format!("Live: {player_count}"))
            .font_mono_500()
            .hex_text_color("#000000")]
    } else {
        [Text::el("Error")
            .with(translation(), vec3(0., 0., 0.))
            .font_mono_500()
            .hex_text_color("#000000")]
    })
    .with(translation(), vec3(0., 0., 0.))
    .hex_background("#ffffff")
}
