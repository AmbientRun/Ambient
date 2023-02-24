use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::{camera::active_camera, transform::translation};
use ambient_element::{ElementComponentExt, Group};
use ambient_std::color::Color;
use ambient_ui::*;
use glam::vec3;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let background = |e| {
        FlowRow(vec![e]).el().with_background(Color::rgba(1., 1., 1., 0.02)).set(fit_vertical(), Fit::None).set(fit_horizontal(), Fit::None)
    };
    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        Dock(vec![
            background(Text::el("First")).set(height(), 30.).set(margin(), Borders::even(10.)),
            background(Text::el("Second bottom")).set(docking(), Docking::Bottom).set(height(), 50.).set(margin(), Borders::even(10.)),
            background(Text::el("Third left")).set(docking(), Docking::Left).set(width(), 70.),
            Dock(vec![background(Text::el("Fourth padding"))])
                .el()
                .set(padding(), Borders::even(10.))
                .set(height(), 70.)
                .with_background(Color::rgba(1., 1., 1., 0.02)),
            background(Text::el("Fill remainder")).set(margin(), Borders::even(30.)),
        ])
        .el()
        .with_background(Color::rgba(1., 1., 1., 0.02))
        .set(translation(), vec3(10., 10., 0.))
        .set(width(), 500.)
        .set(height(), 500.),
    ])
    .el()
    .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
