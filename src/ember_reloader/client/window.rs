use ambient_api::{
    core::{
        messages::{WindowMouseInput, WindowMouseMotion},
        text::{
            components::{font_size, font_style},
            types::FontStyle,
        },
        transform::components::translation,
    },
    prelude::*,
};

#[element_component]
pub fn Window(
    hooks: &mut Hooks,
    title: String,
    visible: bool,
    close: Option<Cb<dyn Fn() + Send + Sync>>,
    child: Element,
) -> Element {
    let (dragging, set_dragging) = hooks.use_state(false);
    let (position, set_position) = hooks.use_state(Vec2::ONE * 100.0);

    hooks.use_runtime_message::<WindowMouseInput>({
        let set_dragging = set_dragging.clone();
        move |_world, event| {
            if event.button == MouseButton::Left.into() && !event.pressed {
                set_dragging(false);
            }
        }
    });

    hooks.use_runtime_message::<WindowMouseMotion>(move |_world, event| {
        if dragging {
            set_position(position + event.delta);
        }
    });

    if !visible {
        return Element::new();
    }

    let title = with_rect(FlowRow::el([
        close
            .map(|close| {
                Button::new(" X ", move |_| close())
                    .style(ButtonStyle::Card)
                    .el()
            })
            .unwrap_or_default(),
        Text::el(title)
            .with_margin_even(4.0)
            .with(font_style(), FontStyle::Bold)
            .with(font_size(), 14.),
    ]))
    .with_background(vec4(0.0, 0.0, 0.0, 0.5))
    .with(fit_horizontal(), Fit::Parent)
    .with_clickarea()
    .on_mouse_input(move |_world, _, input, button| {
        if button == MouseButton::Left {
            set_dragging(input == MouseInput::Pressed);
        }
    })
    .el();

    with_rect(FlowColumn::el([title, child]))
        .with_background(vec4(0.0, 0.0, 0.0, 0.5))
        .with(translation(), vec3(position.x, position.y, -0.001))
}
