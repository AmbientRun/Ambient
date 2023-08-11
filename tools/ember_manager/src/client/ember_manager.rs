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

use crate::embers::ember_manager::{
    self,
    messages::{WasmReload, WasmSetEnabled},
};

use super::{use_hotkey_toggle, use_input_request, Window};

#[element_component]
pub fn EmberManager(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F4);
    use_editor_menu_bar(hooks, "Ember Manager".to_string(), {
        let set_visible = set_visible.clone();
        move || set_visible(!visible)
    });

    FocusRoot::el([Window::el(
        "Ember Manager".to_string(),
        visible,
        Some(cb(move || set_visible(false))),
        EmberManagerInner::el(),
    )])
}

#[element_component]
fn EmberManagerInner(hooks: &mut Hooks) -> Element {
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

    struct Ember {
        name: String,
        enabled: bool,
        description: String,
        client_modules: Vec<EntityId>,
        server_modules: Vec<EntityId>,
    }
    impl Ember {
        fn module_iter(&self) -> impl Iterator<Item = EntityId> + '_ {
            self.client_modules
                .iter()
                .chain(self.server_modules.iter())
                .copied()
        }
    }

    let mut embers = HashMap::new();
    for (id, (on_server, name, enabled, _)) in modules {
        let name = name
            .strip_suffix("_server")
            .or_else(|| name.strip_suffix("_client"))
            .or_else(|| name.strip_prefix("client_"))
            .or_else(|| name.strip_prefix("server_"))
            .unwrap_or(&name)
            .to_string();

        let ember = embers.entry(name.clone()).or_insert_with(|| Ember {
            name: name.clone(),
            enabled: true,
            description: "Lorem ipsum dolor sit amet.".to_string(),
            client_modules: Vec::new(),
            server_modules: Vec::new(),
        });

        if on_server {
            ember.server_modules.push(id);
        } else {
            ember.client_modules.push(id);
        }
        ember.enabled &= enabled;
    }

    let mut embers: Vec<_> = embers.into_values().collect();
    embers.sort_by_key(|ember| ember.name.clone());

    FlowColumn::el(embers.into_iter().map(|ember| {
        let ember_modules = ember.module_iter().collect::<Vec<_>>();

        FlowRow::el([
            ImageFromUrl {
                url: ember_manager::assets::url("construction.png"),
            }
            .el()
            .with(width(), 48.0)
            .with(height(), 48.0),
            FlowColumn::el([
                Checkbox::new(ember.enabled, {
                    to_owned![ember_modules];
                    move |value| {
                        for &id in &ember_modules {
                            WasmSetEnabled::new(id, value).send_server_reliable();
                        }
                    }
                })
                .el(),
                Button::new(FontAwesomeIcon::el(0xf2f1, true), {
                    to_owned![ember_modules];
                    move |_| {
                        for &id in &ember_modules {
                            WasmReload::new(id).send_server_reliable();
                        }
                    }
                })
                .style(ButtonStyle::Flat)
                .el(),
            ]),
            FlowColumn::el([
                FlowRow::el([
                    Text::el(ember.name).with(font_style(), FontStyle::Bold),
                    Text::el("0.0.1 by"),
                    Text::el("Ambient").with(font_style(), FontStyle::Italic),
                ])
                .with(space_between_items(), 4.0),
                Text::el(ember.description),
            ])
            .with(space_between_items(), 8.0),
        ])
        .with(space_between_items(), 8.0)
    }))
    .with(space_between_items(), 4.0)
    .with_margin_even(STREET)
}

// TODO: is there a way to share this?
fn use_editor_menu_bar(
    hooks: &mut Hooks,
    name: String,
    on_click: impl Fn() + Send + Sync + 'static,
) {
    use crate::embers::editor_schema::messages::{
        EditorLoad, EditorMenuBarAdd, EditorMenuBarClick,
    };

    let add = cb({
        let name = name.clone();
        move || EditorMenuBarAdd { name: name.clone() }.send_local_broadcast(false)
    });

    hooks.use_module_message::<EditorLoad>({
        let add = add.clone();
        move |_, _, _| {
            add();
        }
    });

    hooks.use_spawn(move |_| {
        add();
        |_| {}
    });

    hooks.use_module_message::<EditorMenuBarClick>(move |_, _, message| {
        if message.name == name {
            on_click();
        }
    });
}
