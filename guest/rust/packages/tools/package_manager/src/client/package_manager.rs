use crate::ambient_element::use_module_message;
use crate::ambient_element::use_spawn;
use ambient_api::{
    core::{
        package::components::{
            authors, client_modules, description, enabled, is_package, name, server_modules,
            version,
        },
        text::{components::font_style, types::FontStyle},
    },
    element::{use_module_message_effect, use_query},
    prelude::*,
    ui::ImageFromUrl,
};

use crate::packages::{
    input_schema::messages::{InputRelease, InputRequest},
    this::{
        assets,
        messages::{PackageSetEnabled, WasmReload},
    },
};

use super::use_hotkey_toggle;

#[element_component]
pub fn PackageManager(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F4);
    use_editor_menu_bar(hooks, "Package Manager".to_string(), {
        let set_visible = set_visible.clone();
        move || set_visible(!visible)
    });

    FocusRoot::el([Window::el(
        "Package Manager".to_string(),
        visible,
        Some(cb(move || set_visible(false))),
        PackageManagerInner::el(),
    )])
}

#[element_component]
fn PackageManagerInner(hooks: &mut Hooks) -> Element {
    use_module_message_effect::<InputRequest, InputRelease>(hooks, None);

    let packages = use_query(
        hooks,
        (
            is_package(),
            enabled(),
            name(),
            version(),
            authors(),
            client_modules(),
            server_modules(),
        ),
    );

    struct Package {
        enabled: bool,
        name: String,
        version: String,
        authors: Vec<String>,
        description: Option<String>,
        client_modules: Vec<EntityId>,
        server_modules: Vec<EntityId>,
    }

    let mut packages: Vec<_> = packages
        .into_iter()
        .map(
            |(id, (_, enabled, name, version, authors, client_modules, server_modules))| {
                let description = entity::get_component(id, description());

                (
                    id,
                    Package {
                        enabled,
                        name,
                        version,
                        authors,
                        description,
                        client_modules,
                        server_modules,
                    },
                )
            },
        )
        .collect();
    packages.sort_by_key(|(_, package)| package.name.clone());

    FlowColumn::el(packages.into_iter().map(|(id, package)| {
        FlowRow::el([
            ImageFromUrl {
                url: assets::url("construction.png"),
            }
            .el()
            .with(width(), 48.0)
            .with(height(), 48.0),
            FlowColumn::el([
                Checkbox::new(package.enabled, move |value| {
                    PackageSetEnabled { id, enabled: value }.send_server_reliable();
                })
                .el(),
                Button::new(FontAwesomeIcon::el(0xf2f1, true), {
                    let modules: Vec<_> = package
                        .client_modules
                        .iter()
                        .chain(package.server_modules.iter())
                        .copied()
                        .collect();
                    move |_| {
                        for &id in &modules {
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
                    Text::el(package.version),
                    Text::el("by"),
                    Text::el(if package.authors.is_empty() {
                        "No authors specified".to_string()
                    } else {
                        package.authors.join(", ")
                    })
                    .with(font_style(), FontStyle::Italic),
                ])
                .with(space_between_items(), 4.0),
                Text::el(package.description.as_deref().unwrap_or("No description")),
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
    use crate::packages::editor_schema::messages::{
        EditorLoad, EditorMenuBarAdd, EditorMenuBarClick,
    };

    let add = cb({
        let name = name.clone();
        move || EditorMenuBarAdd { name: name.clone() }.send_local_broadcast(false)
    });

    use_module_message::<EditorLoad>(hooks, {
        let add = add.clone();
        move |_, _, _| {
            add();
        }
    });

    use_spawn(hooks, move |_| {
        add();
        |_| {}
    });

    use_module_message::<EditorMenuBarClick>(hooks, move |_, _, message| {
        if message.name == name {
            on_click();
        }
    });
}
