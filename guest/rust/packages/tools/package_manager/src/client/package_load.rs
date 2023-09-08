use std::collections::HashMap;

use ambient_api::{
    core::{
        text::{components::font_style, types::FontStyle},
        wasm::components::{bytecode_from_url, module_name},
    },
    element::{
        use_module_message, use_module_message_effect, use_query, use_state, use_state_with,
    },
    prelude::*,
};

use crate::packages::{
    input_schema::messages::{InputRelease, InputRequest},
    this::messages,
};

use super::use_hotkey_toggle;

#[element_component]
pub fn PackageLoad(_hooks: &mut Hooks) -> Element {
    Group::el([
        PackageLoadDialog::el(),
        PackageView::el(),
        ErrorMessage::el(),
    ])
}

#[element_component]
fn PackageLoadDialog(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F2);
    let close = cb(move || set_visible(false));
    Window::el(
        "Package load".to_string(),
        visible,
        Some(close.clone()),
        PackageLoadDialogInner::el(close),
    )
}

#[element_component]
fn PackageLoadDialogInner(hooks: &mut Hooks, close: Cb<dyn Fn() + Sync + Send>) -> Element {
    use_module_message_effect::<InputRequest, InputRelease>(hooks, None);
    let (url, set_url) = use_state_with(hooks, |_| String::new());

    FlowColumn::el([
        Text::el("Enter package URL:").with_margin_even(STREET),
        TextEditor::new(url, set_url.clone())
            .auto_focus()
            .placeholder(Some("URL"))
            .on_submit(move |url| {
                messages::PackageLoad { url }.send_server_reliable();
                set_url(String::new());
                close();
            })
            .el()
            .with_background(vec4(0.0, 0.0, 0.0, 0.5))
            .with_padding_even(4.0)
            .with(fit_horizontal(), Fit::Parent)
            .with(min_height(), 22.0)
            .with(margin(), vec4(0.0, STREET, STREET, STREET)),
    ])
    .with(min_width(), 600.0)
}

#[element_component]
fn PackageView(hooks: &mut Hooks) -> Element {
    let (msg, set_msg) = use_state(hooks, None);
    use_module_message::<messages::PackageLoadSuccess>(hooks, {
        let set_msg = set_msg.clone();
        move |_, source, msg| {
            if !source.server() {
                return;
            }
            set_msg(Some(msg.clone()));
        }
    });
    Window::el(
        "Package info".to_string(),
        msg.is_some(),
        Some(cb(move || set_msg(None))),
        PackageViewInner::el(msg),
    )
}

#[element_component]
fn PackageViewInner(hooks: &mut Hooks, msg: Option<messages::PackageLoadSuccess>) -> Element {
    use_module_message_effect::<InputRequest, InputRelease>(hooks, None);
    let modules_by_name: HashMap<_, _> = use_query(hooks, (module_name(), bytecode_from_url()))
        .into_iter()
        .map(|(id, (name, url))| (name, (id, url)))
        .collect();

    let msg = msg.unwrap();

    let authors = if msg.authors.is_empty() {
        "Unknown".to_string()
    } else {
        msg.authors.join(", ")
    };

    let subtitle = format!("{} {} by {authors}", msg.name, msg.version);

    fn url_to_name(url: &str) -> String {
        url.rsplit_once('/')
            .map(|(_, name)| name)
            .unwrap_or(url)
            .split_once('.')
            .map(|(name, _)| name)
            .unwrap_or(url)
            .to_string()
    }

    fn render_wasm(url: String, existing_modules: &HashMap<String, (EntityId, String)>) -> Element {
        let name = url_to_name(&url);

        let existing_module = existing_modules.get(&name).cloned();

        FlowRow::el([
            match existing_module.clone() {
                Some((id, existing_url)) => Button::new("Replace existing", {
                    let url = url.clone();
                    move |_| {
                        messages::WasmReplaceBytecodeUrl {
                            id,
                            url: url.clone(),
                        }
                        .send_server_reliable();
                    }
                })
                .style(ButtonStyle::Flat)
                .disabled(url == existing_url)
                .el(),
                None => Element::new(),
            },
            FlowColumn::el([
                Text::el(name).with(align_vertical(), Align::Center),
                Text::el(match &existing_module {
                    Some((_, existing_url)) => {
                        if *existing_url == url {
                            format!("{url} (already loaded)")
                        } else {
                            format!("{existing_url} -> {url}")
                        }
                    }
                    None => url,
                })
                .small_style()
                .with(font_style(), FontStyle::Italic),
            ]),
        ])
        .with(space_between_items(), STREET)
    }

    FlowColumn::el([
        FlowColumn::el([
            Text::el(msg.id).with(font_style(), FontStyle::Bold),
            Text::el(subtitle).with(font_style(), FontStyle::Italic),
        ])
        .with(space_between_items(), 4.0),
        FlowColumn::el([
            Text::el("Client WASM").with(font_style(), FontStyle::Bold),
            FlowColumn::el(
                msg.client_wasms
                    .into_iter()
                    .map(|url| render_wasm(url, &modules_by_name)),
            ),
        ])
        .with(space_between_items(), 4.0),
        FlowColumn::el([
            Text::el("Server WASM").with(font_style(), FontStyle::Bold),
            FlowColumn::el(
                msg.server_wasms
                    .into_iter()
                    .map(|url| render_wasm(url, &modules_by_name)),
            ),
        ])
        .with(space_between_items(), 4.0),
    ])
    .with(space_between_items(), STREET)
    .with_margin_even(STREET)
}

#[element_component]
fn ErrorMessage(hooks: &mut Hooks) -> Element {
    let (reason, set_reason) = use_state(hooks, None);
    use_module_message::<messages::ErrorMessage>(hooks, {
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
        "Package load fail".to_string(),
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
    use_module_message_effect::<InputRequest, InputRelease>(hooks, None);
    FlowColumn::el([Text::el(reason), Button::new("OK", move |_| close()).el()])
        .with(space_between_items(), 4.0)
        .with_margin_even(STREET)
}
