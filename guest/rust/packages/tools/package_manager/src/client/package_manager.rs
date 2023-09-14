use ambient_api::{
    core::{
        package::{components::description, concepts::Package},
        text::{components::font_style, types::FontStyle},
    },
    element::{use_module_message, use_query, use_spawn},
    prelude::*,
    ui::ImageFromUrl,
};

use crate::packages::this::{
    assets,
    messages::{PackageLoadShow, PackageSetEnabled, PackageShow},
};

use super::use_hotkey_toggle;

#[element_component]
pub fn PackageManager(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F4);
    use_editor_menu_bar(hooks, "Package Manager".to_string(), {
        let set_visible = set_visible.clone();
        move || set_visible(!visible)
    });

    Window::el(
        "Package Manager".to_string(),
        visible,
        Some(cb(move || set_visible(false))),
        PackageManagerInner::el(),
    )
}

#[element_component]
fn PackageManagerInner(hooks: &mut Hooks) -> Element {
    let packages = use_query(hooks, Package::as_query());

    let display_packages: Vec<_> = packages
        .into_iter()
        .map(|(id, package)| {
            let description = entity::get_component(id, description());

            DisplayPackage {
                source: DisplayPackageSource::Local { id },
                enabled: package.enabled,
                name: package.name,
                version: package.version,
                authors: package.authors,
                description,
            }
        })
        .collect();

    FlowColumn::el([
        Button::new("Load package", |_| {
            PackageLoadShow.send_local(crate::packages::this::entity())
        })
        .el(),
        PackageList::el(display_packages),
    ])
    .with(space_between_items(), 4.0)
    .with_margin_even(STREET)
}

#[derive(Clone, Debug)]
struct DisplayPackage {
    source: DisplayPackageSource,
    enabled: bool,
    name: String,
    version: String,
    authors: Vec<String>,
    description: Option<String>,
}

#[derive(Clone, Debug)]
enum DisplayPackageSource {
    Local { id: EntityId },
    Remote { url: String },
}

#[element_component]
fn PackageList(_hooks: &mut Hooks, packages: Vec<DisplayPackage>) -> Element {
    let mut packages = packages;
    packages.sort_by_key(|package| package.name.clone());

    FlowColumn::el(packages.into_iter().map(|package| {
        FlowRow::el([
            ImageFromUrl {
                url: assets::url("construction.png"),
            }
            .el()
            .with(width(), 48.0)
            .with(height(), 48.0),
            match &package.source {
                DisplayPackageSource::Local { id } => {
                    let id = *id;
                    FlowColumn::el([Checkbox::new(package.enabled, move |value| {
                        PackageSetEnabled { id, enabled: value }.send_server_reliable();
                    })
                    .el()])
                }
                _ => Element::new(),
            },
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
                match &package.source {
                    DisplayPackageSource::Local { id } => {
                        let id = *id;
                        Button::new("View", move |_| {
                            PackageShow { id }.send_local(crate::packages::this::entity())
                        })
                        .style(ButtonStyle::Flat)
                        .el()
                    }
                    _ => Element::new(),
                },
            ])
            .with(space_between_items(), 8.0),
        ])
        .with(space_between_items(), 8.0)
    }))
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
