use ambient_api::{
    components::core::{
        layout::{fit_horizontal_parent, margin, min_height, space_between_items},
        text::font_style,
    },
    prelude::*,
};

use super::{use_hotkey_toggle, use_input_request, Window};
use crate::messages;

#[element_component]
pub fn EmberLoad(_hooks: &mut Hooks) -> Element {
    FocusRoot::el([EmberLoadDialog::el(), EmberView::el(), ErrorMessage::el()])
}

#[element_component]
fn EmberLoadDialog(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F2);
    let close = cb(move || set_visible(false));
    Window::el(
        "Ember load".to_string(),
        visible,
        Some(close.clone()),
        EmberLoadDialogInner::el(close),
    )
}

#[element_component]
fn EmberLoadDialogInner(hooks: &mut Hooks, close: Cb<dyn Fn() + Sync + Send>) -> Element {
    use_input_request(hooks);
    let (url, set_url) = hooks.use_state_with(|_| String::new());

    FlowColumn::el([
        Text::el("Enter ember URL:").with_margin_even(STREET),
        TextEditor::new(url, set_url.clone())
            .auto_focus()
            .placeholder(Some("URL"))
            .on_submit(move |url| {
                messages::EmberLoad { url }.send_server_reliable();
                set_url(String::new());
                close();
            })
            .el()
            .with_background(vec4(0.0, 0.0, 0.0, 0.5))
            .with_padding_even(4.0)
            .with_default(fit_horizontal_parent())
            .with(min_height(), 22.0)
            .with(margin(), vec4(0.0, STREET, STREET, STREET)),
    ])
}

#[element_component]
fn EmberView(hooks: &mut Hooks) -> Element {
    let (msg, set_msg) = hooks.use_state(None);
    hooks.use_module_message::<messages::EmberLoadSuccess>({
        let set_msg = set_msg.clone();
        move |_, source, msg| {
            if !source.server() {
                return;
            }
            set_msg(Some(msg.clone()));
        }
    });
    Window::el(
        "Ember info".to_string(),
        msg.is_some(),
        Some(cb(move || set_msg(None))),
        EmberViewInner::el(msg),
    )
}

#[element_component]
fn EmberViewInner(hooks: &mut Hooks, msg: Option<messages::EmberLoadSuccess>) -> Element {
    use_input_request(hooks);
    let msg = msg.unwrap();

    let authors = if msg.authors.is_empty() {
        "Unknown".to_string()
    } else {
        msg.authors.join(", ")
    };

    let subtitle = match msg.name {
        Some(name) => format!("{} {}", name, msg.version),
        None => msg.version,
    };
    let subtitle = format!("{subtitle} by {authors}");

    fn url_to_name(url: String) -> String {
        url.rsplit_once('/')
            .map(|(_, name)| name)
            .unwrap_or(&url)
            .split_once('.')
            .map(|(name, _)| name)
            .unwrap_or(&url)
            .to_string()
    }

    FlowColumn::el([
        FlowColumn::el([
            Text::el(msg.id).with(font_style(), "Bold".to_string()),
            Text::el(subtitle).with(font_style(), "Italic".to_string()),
        ])
        .with(space_between_items(), 4.0),
        FlowColumn::el([
            Text::el("Client WASM").with(font_style(), "Bold".to_string()),
            FlowColumn::el(
                msg.client_wasms
                    .into_iter()
                    .map(|url| Text::el(url_to_name(url))),
            ),
        ])
        .with(space_between_items(), 4.0),
        FlowColumn::el([
            Text::el("Server WASM").with(font_style(), "Bold".to_string()),
            FlowColumn::el(
                msg.server_wasms
                    .into_iter()
                    .map(|url| Text::el(url_to_name(url))),
            ),
        ])
        .with(space_between_items(), 4.0),
    ])
    .with(space_between_items(), STREET)
    .with_margin_even(STREET)
}

#[element_component]
fn ErrorMessage(hooks: &mut Hooks) -> Element {
    let (reason, set_reason) = hooks.use_state(None);
    hooks.use_module_message::<messages::ErrorMessage>({
        let set_reason = set_reason.clone();
        move |_, source, msg| {
            if !source.server() {
                return;
            }
            set_reason(Some(msg.reason.clone()));
        }
    });
    let close = cb(move || set_reason(None));
    Window::el(
        "Ember load fail".to_string(),
        reason.is_some(),
        Some(close.clone()),
        ErrorMessageInner::el(reason.unwrap_or_default(), close),
    )
}

#[element_component]
fn ErrorMessageInner(
    hooks: &mut Hooks,
    reason: String,
    close: Cb<dyn Fn() + Send + Sync>,
) -> Element {
    use_input_request(hooks);
    FlowColumn::el([Text::el(reason), Button::new("OK", move |_| close()).el()])
        .with(space_between_items(), 4.0)
        .with_margin_even(STREET)
}
