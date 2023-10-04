use ambient_api::{
    core::{
        rect::components::background_color, rendering::components::color,
        text::components::font_size, transform::components::translation,
    },
    prelude::*,
    ui::ImageFromUrl,
};
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
    FlowRow::el(if let Some(player_count) = player_count {
        [
            ImageFromUrl {
                url: packages::this::assets::url("white_dot.png"), // TODO: replace with a CIRCLE!
            }
            .el()
            .with(color(), vec4(0.055, 0.345, 0.235, 1.))
            .with(width(), 5.)
            .with(height(), 5.),
            Text::el(format!("LIVE: {player_count}"))
                .font_mono_500()
                .hex_text_color("#000000"),
        ]
    } else {
        [
            ImageFromUrl {
                url: packages::this::assets::url("white_dot.png"), // TODO: replace with a CIRCLE!
            }
            .el()
            .with(color(), vec4(0.839, 0.357, 0.267, 1.))
            .with(width(), 5.)
            .with(height(), 5.),
            Text::el("Error")
                .with(translation(), vec3(0., 0., 0.))
                .font_mono_500()
                .hex_text_color("#000000"),
        ]
    })
    .with(translation(), vec3(5.5, 5.5, 0.))
    // .with(background_color(), Vec3::splat(0.98).extend(1.))
    .with_background(Vec3::splat(0.98).extend(1.))
    .with(fit_vertical(), Fit::None)
    .with(fit_horizontal(), Fit::Children)
    .with(align_horizontal(), Align::Begin)
    .with(align_vertical(), Align::Center)
    // .with(width(), 100.)
    .with(height(), 12.)
    .with(space_between_items(), 6.)
    .with(padding(), vec4(1., 3., 1., 6.))
}
