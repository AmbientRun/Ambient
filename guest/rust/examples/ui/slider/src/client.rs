use ambient_api::prelude::*;
use ambient_element::{element_component, Element, ElementComponentExt, Hooks};
use ambient_guest_bridge::components::layout::space_between_items;
use ambient_ui_components::{
    default_theme::STREET,
    editor::{IntegerSlider, Slider},
    layout::FlowColumn,
    FocusRoot, UIExt,
};

#[element_component]
fn App(hooks: &mut Hooks) -> Element {
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
    .with(space_between_items(), STREET)
    .with_padding_even(STREET)
}

#[main]
pub async fn main() -> EventResult {
    App.el().spawn_interactive();

    EventOk
}
