use ambient_api::{core::layout::components::space_between_items, element::use_state, prelude::*};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
    let (f32_value, set_f32_value) = use_state(hooks, 0.);
    let (f32_exp_value, set_f32_exp_value) = use_state(hooks, 0.1);
    let (i32_value, set_i32_value) = use_state(hooks, 0);

    FlowColumn::el([
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
    ])
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
