//! Implements a window with a title bar and a child element. Can be moved around.

use ambient_cb::{cb, Cb};
use ambient_element::{
    element_component, use_runtime_message, use_state, Element, ElementComponentExt, Hooks,
};
use ambient_guest_bridge::core::{
    layout::{components::fit_horizontal, types::Fit},
    messages::{WindowMouseInput, WindowMouseMotion},
    text::{
        components::{font_size, font_style},
        types::FontStyle,
    },
    transform::components::translation,
    ui::components::focusable,
};
use ambient_shared_types::MouseButton;
use glam::{vec3, vec4, Vec2};

use crate::{
    button::{Button, ButtonStyle},
    clickarea::MouseInput,
    layout::{FlowColumn, FlowRow},
    text::Text,
    with_rect, UIExt,
};

/// A chance to style the window
#[derive(Debug, Clone)]
pub struct WindowStyle {
    /// Body style
    pub body: Cb<dyn Fn(Element) -> Element + Send + Sync>,
    /// Title bar style
    pub title_bar: Cb<dyn Fn(Element) -> Element + Send + Sync>,
    /// Title text style
    pub title_text: Cb<dyn Fn(Element) -> Element + Send + Sync>,
}
impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            body: cb(|e| e),
            title_bar: cb(|e| e),
            title_text: cb(|e| e),
        }
    }
}

#[element_component]
/// A window with a title bar and a child element. Can be moved around.
pub fn Window(
    hooks: &mut Hooks,
    /// The title of the window.
    title: String,
    /// A callback to be called when the window requests to be closed.
    /// If this is `None`, the window will not have a close button.
    /// This callback should update `visible` to `false`.
    close: Option<Cb<dyn Fn() + Send + Sync>>,
    /// Whether the window is visible.
    visible: bool,
    /// An optional chance to style the body of the window.
    style: Option<WindowStyle>,
    /// The child element.
    child: Element,
) -> Element {
    let (dragging, set_dragging) = use_state(hooks, false);
    let (position, set_position) = use_state(hooks, Vec2::ONE * 100.0);

    use_runtime_message::<WindowMouseInput>(hooks, {
        let set_dragging = set_dragging.clone();
        move |_world, event| {
            if event.button == u32::from(MouseButton::Left) && !event.pressed {
                set_dragging(false);
            }
        }
    });

    use_runtime_message::<WindowMouseMotion>(hooks, move |_world, event| {
        if dragging {
            set_position(position + event.delta);
        }
    });

    if !visible {
        return Element::new();
    }

    let style = style.unwrap_or_default();

    let title = (style.title_bar)(
        with_rect(FlowRow::el([
            close
                .map(|close| {
                    Button::new(" X ", move |_| close())
                        .style(ButtonStyle::Card)
                        .el()
                })
                .unwrap_or_default(),
            (style.title_text)(
                Text::el(title)
                    .with_margin_even(4.0)
                    .with(font_style(), FontStyle::Bold)
                    .with(font_size(), 14.),
            ),
        ]))
        .with_background(vec4(0.0, 0.0, 0.0, 0.5))
        .with(fit_horizontal(), Fit::Parent),
    )
    .with(focusable(), hooks.instance_id().to_string())
    .with_clickarea()
    .on_mouse_input(move |_world, _, input, button| {
        if button == MouseButton::Left {
            set_dragging(input == MouseInput::Pressed);
        }
    })
    .el();

    (style.body)(
        with_rect(FlowColumn::el([title, child])).with_background(vec4(0.0, 0.0, 0.0, 0.5)),
    )
    .with(translation(), vec3(position.x, position.y, -0.001))
}
