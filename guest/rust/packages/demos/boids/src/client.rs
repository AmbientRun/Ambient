use ambient_api::core::rendering::components::{color, transparency_group};
use ambient_api::core::text::components::font_size;
use ambient_api::core::transform::components::{local_to_world, translation};
use ambient_api::prelude::*;
use packages::this::components::*;

#[main]
pub fn main() {
    BoidNeighbours::el().spawn_interactive();
}

#[element_component]
fn BoidNeighbours(hooks: &mut Hooks) -> Element {
    let cameras = ambient_api::element::use_query(hooks, is_boid_camera());

    if let Some((camera, _)) = cameras.first() {
        let Some(_camera_inv_view) =
            ambient_api::element::use_entity_component(hooks, *camera, local_to_world())
        else {
            return Element::new();
        };

        let (_, _, _camera_translation) = _camera_inv_view.to_scale_rotation_translation();

        let boids = ambient_api::element::use_query(hooks, (translation(), boid_neighbour_count()));

        Group::el(boids.into_iter().map(move |(_boid, (pos, bns))| {
            let player_nameplate_screen_pos =
                camera::world_to_screen(*camera, pos + vec3(0., 0., 2.));
            let player_nameplate_ui_pos = vec3(
                player_nameplate_screen_pos.x as f32,
                player_nameplate_screen_pos.y as f32,
                0.0,
            );

            FlowColumn::el([
                // match plr == &player::get_local() {
                //     true => ImageFromUrl {
                //         url: packages::this::assets::url("ok_star.png"),
                //     }
                //     .el()
                //     .with(width(), 24.0)
                //     .with(height(), 24.0),
                //     false => Element::new(),
                // },
                // FlowRow::el([Text::el(floatemp_to_string(*player_temp))
                //     .with(color(), C_ALLBLACK.extend(1.))
                //     .with(font_size(), fsize * 0.45 * nameplate_scale)
                //     .font_body_500()]), // .with_background(C_ALLWHITE.extend(1.))
                // FlowRow::el([Text::el(format!("{}", bns))
                //     .with(color(), C_ALLBLACK.extend(1.))
                //     .with(font_size(), fsize * 0.65 * nameplate_scale)
                //     .font_body_500()]), // .with_background(C_ALLWHITE.extend(1.))
                Text::el(format!("{}", bns))
                    .with(font_size(), (10 + bns * 2) as f32)
                    .with(color(), vec4(1., 1., 1., 0.5 + bns as f32 * 0.05))
                    .with(transparency_group(), 2),
            ])
            .with(fit_vertical(), Fit::None)
            .with(fit_horizontal(), Fit::None)
            .with(align_horizontal(), Align::Center)
            .with(align_vertical(), Align::End)
            .with(width(), 20.)
            .with(height(), 10.)
            .with(
                translation(),
                player_nameplate_ui_pos + vec3(-10., -10., 0.),
            )
        }))
    } else {
        Element::new()
    }
}
