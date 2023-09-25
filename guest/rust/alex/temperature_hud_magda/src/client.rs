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
            ImageFromUrl {
                url: packages::this::assets::url("ok_star.png"),
            }
            .el()
            .with(width(), 24.0)
            .with(height(), 24.0),
            FlowRow::el([Text::el(floatemp_to_string(*player_temp))
                .with(color(), C_ALLBLACK.extend(1.))
                .with(font_size(), fsize * 0.45)
                .with(font_family(), FONT_PATH_CHANGE_THIS.into())])
            .with_background(C_ALLWHITE.extend(1.)),
            FlowRow::el([Text::el(format!("{}", uid))
                .with(color(), C_ALLBLACK.extend(1.))
                .with(font_size(), fsize * 0.65)
                .with(font_family(), FONT_PATH_CHANGE_THIS.into())])
            .with_background(C_ALLWHITE.extend(1.)),
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

#[allow(dead_code)]
const C_VINTER_HIMMEL_5: Vec3 = Vec3::new(0.804, 0.804, 0.804); //#CDCDCD
#[allow(dead_code)]
const C_NASTAN_SVART_5: Vec3 = Vec3::new(0.1451, 0.1451, 0.1451); //#252525
const C_ALLBLACK: Vec3 = Vec3::new(0., 0., 0.); //#000
const C_ALLWHITE: Vec3 = Vec3::new(1., 1., 1.); //#FFF

const C_TEMPBAR_HOT: Vec3 = Vec3::new(0.749, 0.129, 0.149);
const C_TEMPBAR_MID: Vec3 = Vec3::new(0.851, 0.851, 0.851);
const C_TEMPBAR_COLD: Vec3 = Vec3::new(0.592, 0.780, 0.851);
const HOT_PERCENTILE: f32 = 0.0625;
const COLD_PERCENTILE: f32 = 0.25;
const TEMPBAR_COLDEST_TEMP: f32 = 0.;
const TEMPBAR_HOTTEST_TEMP: f32 = 60.;

fn remap(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
}

// DISPLAYS TEMPERATURE of player
#[element_component]
pub fn TemperatureDisplayUI(hooks: &mut Hooks) -> Element {
    let (player_temp, _) =
        ambient_api::element::use_entity_component(hooks, player::get_local(), temperature());

    let screen_size = entity::get_component(entity::resources(), window_physical_size()).unwrap();

    if let Some(player_temp) = player_temp {
        let fsize = screen_size.y as f32 * 0.05;
        let tb_centerpos = (vec2(0.1, 0.5) * screen_size.as_vec2()).extend(0.0);
        let tb_size = (vec2(0.015, 0.22) * screen_size.as_vec2()).extend(0.0);
        let tb_hot_centerpos = tb_centerpos - vec3(0., tb_size.y * (1. - HOT_PERCENTILE) * 0.5, 0.);
        let tb_hot_size = tb_size * vec3(1., HOT_PERCENTILE, 1.);
        let tb_cold_centerpos =
            tb_centerpos + vec3(0., tb_size.y * (1. - COLD_PERCENTILE) * 0.5, 0.);
        let tb_cold_size = tb_size * vec3(1., COLD_PERCENTILE, 1.);
        let tb_tempdotsize = Vec2::splat(tb_size.x * 0.8);
        let tb_tempdotpos = tb_centerpos
            + vec3(
                -tb_tempdotsize.x * 0.5, // centering
                remap(
                    player_temp,
                    TEMPBAR_COLDEST_TEMP,
                    TEMPBAR_HOTTEST_TEMP,
                    tb_size.y * 0.5,
                    tb_size.y * -0.5,
                ),
                0.,
            );
        Group::el(vec![
            FlowRow::el([
                ImageFromUrl {
                    url: packages::this::assets::url("white_dot.png"), // TODO: replace with a CIRCLE!
                }.el()
                    .with(width(),tb_tempdotsize.x)
                    .with(height(),tb_tempdotsize.y)
                    .with(color(), C_ALLBLACK.extend(1.)),
                Text::el(floatemp_to_string(player_temp))
                    .with(font_size(), fsize)
                    .with(
                        font_family(),
                        "https://jetsam.droqen.com/2023-0918-ambient-game-font-test/ABCDiatype-Regular.otf"
                            .into(),
                    )
                    .with(color(), C_ALLBLACK.extend(1.)),
                match player_temp < 25.0 {
                    false => Text::el(""),
                    true => ImageFromUrl {
                        url: packages::this::assets::url("freezing_skull.png"),
                    }
                    .el()
                        .with(width(), 24.0)
                        .with(height(), 24.0),
                },
            ])
            .with(translation(), tb_tempdotpos)
            .with(align_vertical(), Align::Center)
            .with(space_between_items(), 10.0)
            ,
            Rectangle::el()
                .with(translation(), tb_hot_centerpos - tb_hot_size * 0.5 + vec3(0., 0., 0.01))
                .with(width(), tb_hot_size.x)
                .with(height(), tb_hot_size.y)
                .with(color(), C_TEMPBAR_HOT.extend(1.))
            ,
            Rectangle::el()
                .with(translation(), tb_centerpos - tb_size * 0.5 + vec3(0., 0., 0.02))
                .with(width(), tb_size.x)
                .with(height(), tb_size.y)
                .with(color(), C_TEMPBAR_MID.extend(1.))
            ,
            Rectangle::el()
                .with(translation(), tb_cold_centerpos - tb_cold_size * 0.5 + vec3(0., 0., 0.01))
                .with(width(), tb_cold_size.x)
                .with(height(), tb_cold_size.y)
                .with(color(), C_TEMPBAR_COLD.extend(1.))
            ,

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
