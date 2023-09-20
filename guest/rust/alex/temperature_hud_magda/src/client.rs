use ambient_api::{
    core::{
        app::components::window_physical_size,
        player::components::user_id,
        rect::components::background_color,
        rendering::components::color,
        text::components::{font_family, font_size},
        transform::components::translation,
    },
    prelude::*,
    ui::ImageFromUrl,
};

use packages::temperature::components::temperature;

const FONT_PATH_CHANGE_THIS: &str =
    "https://jetsam.droqen.com/2023-0918-ambient-game-font-test/ABCDiatype-Regular.otf";

#[main]
pub fn main() {
    TemperatureDisplayUI::el().spawn_interactive();

    run_async(async {
        let _ = entity::wait_for_component(
            packages::this::entity(),
            packages::this::components::active_camera(),
        )
        .await;
        if let Some(camera) = entity::get_component(
            packages::this::entity(),
            packages::this::components::active_camera(),
        ) {
            NameplateUI::el(camera).spawn_interactive();
        } else {
            panic!("No camera even after await");
        }
    });
}

// DISPLAYS NAMEPLATE & TEMP above players' heads
#[element_component]
pub fn NameplateUI(hooks: &mut Hooks, camera: EntityId) -> Element {
    let screen_size = entity::get_component(entity::resources(), window_physical_size()).unwrap();
    let players = ambient_api::element::use_query(hooks, (user_id(), translation(), temperature()));
    let fsize = screen_size.y as f32 * 0.05;
    Group::el(players.iter().map(move |(_plr, (uid, pos, player_temp))| {
        FlowColumn::el([
            FlowRow::el([Text::el(floatemp_to_string(*player_temp))
                .with(color(), C_NASTAN_SVART_5.extend(1.))
                .with(font_size(), fsize * 0.45)
                .with(font_family(), FONT_PATH_CHANGE_THIS.into())])
            .with_background(C_VINTER_HIMMEL_5.extend(1.))
            .with(align_horizontal(), Align::Center),
            FlowRow::el([Text::el(format!("PLAYER_{}", uid))
                .with(color(), C_NASTAN_SVART_5.extend(1.))
                .with(font_size(), fsize * 0.65)
                .with(font_family(), FONT_PATH_CHANGE_THIS.into())])
            .with_background(C_VINTER_HIMMEL_5.extend(1.))
            .with(align_horizontal(), Align::Center),
        ])
        .with(align_horizontal(), Align::Center)
        .with(align_vertical(), Align::End)
        .with(width(), 400.0)
        .with(height(), fsize * 0.15)
        .with(
            translation(),
            camera::world_to_screen(camera, *pos + vec3(0., 0., 2.)).extend(0.)
                + vec3(-200.0, 0., 0.),
        )
    }))
}

const C_VINTER_HIMMEL_5: Vec3 = Vec3::new(0.804, 0.804, 0.804); //#CDCDCD
const C_NASTAN_SVART_5: Vec3 = Vec3::new(0.1451, 0.1451, 0.1451); //#252525

// DISPLAYS TEMPERATURE of player
#[element_component]
pub fn TemperatureDisplayUI(hooks: &mut Hooks) -> Element {
    let (player_temp, _) =
        ambient_api::element::use_entity_component(hooks, player::get_local(), temperature());

    let screen_size = entity::get_component(entity::resources(), window_physical_size()).unwrap();

    if let Some(player_temp) = player_temp {
        let fsize = screen_size.y as f32 * 0.05;
        Group::el(vec![
            FlowRow::el([
                Text::el(floatemp_to_string(player_temp))
                    .with(font_size(), fsize)
                    .with(color(), C_NASTAN_SVART_5.extend(1.0))
                    .with(
                        font_family(),
                        "https://jetsam.droqen.com/2023-0918-ambient-game-font-test/ABCDiatype-Regular.otf"
                            .into(),
                    ),
                ImageFromUrl {
                    url: "https://commons.wikimedia.org/w/index.php?title=Special:Redirect/file/Bucephala-albeola-010.jpg"
                        .to_string(),
                }
                    .el(),
            ])
            .with(translation(), vec3(screen_size.x as f32 * 0.1, (screen_size.y as f32-fsize)*0.5, 0.)),

            // FlowRow::el([
            //     background(Text::el("Align")),
            //     background(Text::el("Center").with(font_size(), 30.)),
            // ])
            // .with_background(vec4(0.1, 0.1, 0.1, 1.))
            // .with(fit_vertical(), Fit::None)
            // .with(fit_horizontal(), Fit::None)
            // .with(align_horizontal(), Align::Center)
            // .with(align_vertical(), Align::Center)
            // .with(width(), 200.)
            // .with(height(), 70.)
            // .with_padding_even(10.)
            // .with(space_between_items(), 5.),

            // .with_background(vec4(0.1, 0.1, 0.1, 1.))
            // .with(fit_vertical(), Fit::Children)
            // .with(fit_horizontal(), Fit::Children)
            // .with_padding_even(10.),

            // Text::el(format!(
            //     "{},{}°C",
            //     degree_ones, degree_tenths
            // ))
            //     .with(translation(), vec3(50., 70., 0.))
            //     .with(font_size(), 20.0)
            //     .with(color(), C_NASTAN_SVART_5.extend(1.0)),

        ])
    } else {
        Group::el([])
    }
}

fn floatemp_to_string(floatemp: f32) -> String {
    let decitemp = (floatemp * 10.).floor() as u32;
    println!("{},{}°C", decitemp / 10, decitemp % 10);
    format!("{},{}°C", decitemp / 10, decitemp % 10)
}
