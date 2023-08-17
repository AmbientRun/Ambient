use std::collections::HashMap;

use ambient_api::{
    core::{
        text::{components::font_style, types::FontStyle},
        wasm::components::{
            bytecode_from_url, is_module, is_module_on_server, module_enabled, module_name,
        },
    },
    prelude::*,
    ui::ImageFromUrl,
};

use crate::packages::package_manager::{
    self,
    messages::{WasmReload, WasmSetEnabled},
};

use super::{use_hotkey_toggle, use_input_request, Window};

#[element_component]
pub fn PackageManager(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F4);

    FocusRoot::el([Window::el(
        "Package Manager".to_string(),
        visible,
        Some(cb(move || set_visible(false))),
        PackageManagerInner::el(),
    )])
}

#[element_component]
fn PackageManagerInner(hooks: &mut Hooks) -> Element {
    use_input_request(hooks);

    let modules: Vec<_> = hooks
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

    struct Package {
        name: String,
        enabled: bool,
        description: String,
        client_modules: Vec<EntityId>,
        server_modules: Vec<EntityId>,
    }
    impl Package {
        fn module_iter(&self) -> impl Iterator<Item = EntityId> + '_ {
            self.client_modules
                .iter()
                .chain(self.server_modules.iter())
                .copied()
        }
    }

    let mut packages = HashMap::new();
    for (id, (on_server, name, enabled, _)) in modules {
        let name = name
            .strip_suffix("_server")
            .or_else(|| name.strip_suffix("_client"))
            .or_else(|| name.strip_prefix("client_"))
            .or_else(|| name.strip_prefix("server_"))
            .unwrap_or(&name)
            .to_string();

        let package = packages.entry(name.clone()).or_insert_with(|| Package {
            name: name.clone(),
            enabled: true,
            description: "Lorem ipsum dolor sit amet.".to_string(),
            client_modules: Vec::new(),
            server_modules: Vec::new(),
        });

        if on_server {
            package.server_modules.push(id);
        } else {
            package.client_modules.push(id);
        }
        package.enabled &= enabled;
    }

    let mut packages: Vec<_> = packages.into_values().collect();
    packages.sort_by_key(|package| package.name.clone());

    FlowColumn::el(packages.into_iter().map(|package| {
        let package_modules = package.module_iter().collect::<Vec<_>>();

        FlowRow::el([
            ImageFromUrl {
                url: package_manager::assets::url("construction.png"),
            }
            .el()
            .with(width(), 48.0)
            .with(height(), 48.0),
            FlowColumn::el([
                Checkbox::new(package.enabled, {
                    to_owned![package_modules];
                    move |value| {
                        for &id in &package_modules {
                            WasmSetEnabled::new(id, value).send_server_reliable();
                        }
                    }
                })
                .el(),
                Button::new(FontAwesomeIcon::el(0xf2f1, true), {
                    to_owned![package_modules];
                    move |_| {
                        for &id in &package_modules {
                            WasmReload::new(id).send_server_reliable();
                        }
                    }
                })
                .style(ButtonStyle::Flat)
                .el(),
            ]),
            FlowColumn::el([
                FlowRow::el([
                    Text::el(package.name).with(font_style(), FontStyle::Bold),
                    Text::el("0.0.1 by"),
                    Text::el("Ambient").with(font_style(), FontStyle::Italic),
                ])
                .with(space_between_items(), 4.0),
                Text::el(package.description),
            ])
            .with(space_between_items(), 8.0),
        ])
        .with(space_between_items(), 8.0)
    }))
    .with(space_between_items(), 4.0)
    .with_margin_even(STREET)
}
