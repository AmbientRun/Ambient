use ambient_api::{
    core::{
        text::{components::font_style, types::FontStyle},
        wasm::components::{
            bytecode_from_url, is_module, is_module_on_server, module_enabled, module_name,
        },
    },
    prelude::*,
};

use crate::packages::package_manager::messages::{WasmReload, WasmSetEnabled};

use super::{use_hotkey_toggle, use_input_request, Window};

#[element_component]
pub fn WasmManager(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F3);

    FocusRoot::el([Window::el(
        "WASM Manager".to_string(),
        visible,
        Some(cb(move || set_visible(false))),
        WasmManagerInner::el(),
    )])
}

#[element_component]
fn WasmManagerInner(hooks: &mut Hooks) -> Element {
    use_input_request(hooks);

    let mut modules: Vec<_> = hooks
        .use_query((
            is_module(),
            module_name(),
            module_enabled(),
            bytecode_from_url(),
        ))
        .into_iter()
        .map(|(id, (_, name, enabled, url))| {
            (
                id,
                (
                    entity::has_component(id, is_module_on_server()),
                    name,
                    enabled,
                    url,
                ),
            )
        })
        .collect();
    modules.sort_by_key(|(_, (is_server, name, _, _))| (*is_server, name.clone()));

    let (server_modules, client_modules): (Vec<_>, Vec<_>) = modules
        .into_iter()
        .partition(|(_, (is_server, _, _, _))| *is_server);

    fn render_modules(
        heading: String,
        modules: Vec<(EntityId, (bool, String, bool, String))>,
    ) -> Element {
        FlowColumn::el(Iterator::chain(
            [Text::el(heading).with(font_style(), FontStyle::Bold)].into_iter(),
            modules.into_iter().map(|(id, (_, name, enabled, url))| {
                FlowRow::el([
                    Checkbox::new(enabled, move |value| {
                        WasmSetEnabled::new(id, value).send_server_reliable();
                    })
                    .el(),
                    Button::new(FontAwesomeIcon::el(0xf2f1, true), move |_| {
                        WasmReload::new(id).send_server_reliable();
                    })
                    .style(ButtonStyle::Flat)
                    .el(),
                    FlowColumn::el([
                        Text::el(name),
                        Text::el(url)
                            .small_style()
                            .with(font_style(), FontStyle::Italic),
                    ]),
                ])
                .with(space_between_items(), 4.0)
            }),
        ))
        .with(space_between_items(), 4.0)
    }

    FlowRow::el([
        render_modules("Server modules".to_string(), server_modules),
        render_modules("Client modules".to_string(), client_modules),
    ])
    .with_margin_even(STREET)
    .with(space_between_items(), STREET)
}
