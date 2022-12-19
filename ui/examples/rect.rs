use elements::{app::App, cameras::UICamera, ecs::World};
use elements_core::camera::active_camera;
use elements_element::{ElementComponentExt, Group};
use elements_std::color::Color;
use elements_ui::{
    layout::{height, width}, *
};

fn init(world: &mut World) {
    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        FlowColumn(vec![
            Rectangle.el().set(width(), 100.).set(height(), 100.),
            Rectangle
                .el()
                .set(width(), 200.)
                .set(height(), 100.)
                .set(background_color(), Color::rgba(1., 0., 0., 1.))
                .set(border_radius(), Corners::even(10.))
                .set(border_thickness(), 3.)
                .set(border_color(), Color::rgba(1., 1., 1., 1.)),
        ])
        .el()
        .set(space_between_items(), 5.),
    ])
    .el()
    .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    App::run_ui(init);
}
