use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Group, Hooks};
use ambient_ui::*;

#[derive(Debug, Clone)]
struct Example;
impl ElementComponent for Example {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (f32_value, set_f32_value) = hooks.use_state(0.);
        let (f32_exp_value, set_f32_exp_value) = hooks.use_state(0.1);
        let (i32_value, set_i32_value) = hooks.use_state(0);

        FocusRoot::el([FlowColumn::el([
            Slider {
                value: f32_value,
                on_change: Some(set_f32_value),
                min: 0.,
                max: 100.,
                width: 100.,
                logarithmic: false,
                round: Some(2),
                suffix: Some("%"),
            }
            .el(),
            Slider {
                value: f32_exp_value,
                on_change: Some(set_f32_exp_value),
                min: 0.1,
                max: 1000.,
                width: 100.,
                logarithmic: true,
                round: Some(2),
                suffix: None,
            }
            .el(),
            IntegerSlider {
                value: i32_value,
                on_change: Some(set_i32_value),
                min: 0,
                max: 100,
                width: 100.,
                logarithmic: false,
                suffix: None,
            }
            .el(),
        ])])
        .set(space_between_items(), STREET)
        .set(padding(), Borders::even(STREET))
    }
}

async fn init(app: &mut App) {
    let world = &mut app.world;
    Group(vec![UICamera.el().set(active_camera(), 0.), Example.el()]).el().spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
