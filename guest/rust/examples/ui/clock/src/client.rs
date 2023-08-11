use ambient_api::{
    core::{
        layout::components::{height, width},
        rect::components::{
            background_color, border_color, border_radius, border_thickness, line_from, line_to,
            line_width,
        },
        transform::components::translation,
    },
    prelude::*,
};
use std::f32::consts::PI;

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let clock_r = 100.;
    let second_r = 70.;
    let size_info = hooks.use_window_logical_resolution();
    let center_x = size_info.x as f32 / 2.;
    let center_y = size_info.y as f32 / 2.;
    let (now, set_now) = hooks.use_state(game_time());
    let (x, set_x) = hooks.use_state(size_info.x as f32 / 2.);
    let (y, set_y) = hooks.use_state(size_info.y as f32 / 2. - second_r);
    let (phase, set_phase) = hooks.use_state(PI / 30.);
    hooks.use_frame(move |_world| {
        let latest = game_time();
        if latest - now > Duration::from_secs_f32(1.0) {
            set_now(latest);
            set_phase({
                if phase + PI / 30.0 > PI * 2.0 {
                    phase + PI / 30.0 - PI * 2.0
                } else {
                    phase + PI / 30.0
                }
            });
            set_x(center_x + second_r * (phase.sin()));
            set_y(center_y - second_r * (phase.cos()));
            println!("x: {}, y: {}", x, y);
        }
    });
    Group::el([
        Rectangle
            .el()
            .with(width(), clock_r * 2.)
            .with(height(), clock_r * 2.)
            .with(
                translation(),
                vec3(center_x - clock_r, center_y - clock_r, 0.01),
            )
            .with(border_color(), vec4(0.5, 0.5, 0.5, 1.))
            .with(border_thickness(), 4.)
            .with(border_radius(), Vec4::ONE * clock_r),
        Line.el()
            .with(line_from(), vec3(center_x, center_y, 0.0))
            .with(line_to(), vec3(x, y, 0.0))
            .with(line_width(), 4.)
            .with(background_color(), vec4(0.5, 0.5, 0.5, 1.)),
    ])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
