use std::{collections::HashSet, fmt};

use ambient_api::{
    core::{
        package::{
            components::{description, for_playables, id, is_package},
            concepts::Package as PackageConcept,
        },
        rect::components::background_color,
        text::{components::font_style, types::FontStyle},
    },
    element::{
        use_effect, use_entity_component, use_module_message, use_query, use_spawn, use_state,
    },
    prelude::*,
    ui::ImageFromUrl,
};

use crate::{
    packages::{
        self,
        this::{
            assets,
            messages::{
                PackageLoad, PackageLoadShow, PackageRemoteRequest, PackageRemoteResponse,
                PackageSetEnabled, PackageShow,
            },
        },
    },
    shared::PackageJson,
};

use super::{use_hotkey_toggle, window_style};

#[element_component]
pub fn PackageManager(hooks: &mut Hooks) -> Element {
    let mod_manager_for = use_entity_component(
        hooks,
        packages::this::entity(),
        packages::this::components::mod_manager_for(),
    );

    let title = if mod_manager_for.is_some() {
        "Mod Manager".to_string()
    } else {
        "Package Manager".to_string()
    };

    let (visible, set_visible) = use_hotkey_toggle(hooks, VirtualKeyCode::F4);
    use_editor_menu_bar(hooks, title.clone(), {
        let set_visible = set_visible.clone();
        move || set_visible(!visible)
    });

    Window {
        title: title.clone(),
        visible,
        close: Some(cb(move || set_visible(false))),
        style: Some(window_style()),
        child: if let Some(mod_manager_for) = mod_manager_for {
            ModManagerInner::el(mod_manager_for)
        } else {
            PackageManagerInner::el()
        }
        .with(space_between_items(), 4.0)
        .with_margin_even(STREET),
    }
    .el()
}

#[element_component]
fn ModManagerInner(_hooks: &mut Hooks, mod_manager_for: EntityId) -> Element {
    FlowColumn::el([
        Text::el("Local").header_style(),
        PackagesLocal::el(Some(mod_manager_for)),
        Text::el("Remote").header_style(),
        PackagesRemote::el(),
    ])
}

#[element_component]
fn PackageManagerInner(_hooks: &mut Hooks) -> Element {
    #[derive(PartialEq, Default, Clone, Debug)]
    enum ListTab {
        #[default]
        Local,
        Remote,
    }
    impl fmt::Display for ListTab {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    ListTab::Local => "Local",
                    ListTab::Remote => "Remote",
                }
            )
        }
    }

    Tabs::new()
        .with_tab(ListTab::Local, || PackagesLocal::el(None))
        .with_tab(ListTab::Remote, PackagesRemote::el)
        .el()
}

#[element_component]
fn PackagesLocal(hooks: &mut Hooks, mod_manager_for: Option<EntityId>) -> Element {
    let packages = use_query(hooks, PackageConcept::as_query());

    let mod_manager_for = match mod_manager_for {
        Some(mod_manager_for) => match entity::get_component(mod_manager_for, id()) {
            Some(id) => Some(id),
            None => return Text::el("Could not get ID of main package to mod"),
        },
        None => None,
    };

    let display_packages: Vec<_> = packages
        .into_iter()
        .filter(|(id, _package)| {
            if let Some(mod_manager_for) = &mod_manager_for {
                if let Some(for_playables) = entity::get_component(*id, for_playables()) {
                    for_playables.contains(mod_manager_for)
                } else {
                    false
                }
            } else {
                true
            }
        })
        .map(|(id, package)| {
            let description = entity::get_component(id, description());

            DisplayPackage {
                source: DisplayPackageSource::Local {
                    id,
                    enabled: package.enabled,
                },
                name: package.name,
                version: package.version,
                authors: package.authors,
                description,
            }
        })
        .collect();

    PackageList::el(display_packages)
}

#[derive(Clone, Debug)]
enum PackagesState {
    Loading,
    Loaded(Vec<PackageJson>),
    Error(String),
}
fn use_remote_packages(hooks: &mut Hooks) -> PackagesState {
    let (remote_packages, set_remote_packages) = use_state(hooks, PackagesState::Loading);

    use_effect(hooks, (), move |_, _| {
        PackageRemoteRequest.send_server_reliable();
        |_| {}
    });

    use_module_message::<PackageRemoteResponse>(hooks, move |_, ctx, msg| {
        if !ctx.server() {
            return;
        }

        if let Some(error) = &msg.error {
            set_remote_packages(PackagesState::Error(error.to_string()));
            return;
        }

        let packages_json = msg
            .packages
            .iter()
            .map(|p| serde_json::from_str::<PackageJson>(p))
            .collect::<Result<Vec<_>, _>>();

        match packages_json {
            Ok(packages_json) => set_remote_packages(PackagesState::Loaded(packages_json)),
            Err(error) => {
                set_remote_packages(PackagesState::Error(format!(
                    "Failed to parse packages: {}",
                    error
                )));
            }
        };
    });

    remote_packages
}

#[element_component]
fn PackagesRemote(hooks: &mut Hooks) -> Element {
    let remote_packages = use_remote_packages(hooks);
    let loaded_packages = use_query(hooks, (is_package(), id()));

    let loaded_package_ids: HashSet<String> =
        HashSet::from_iter(loaded_packages.into_iter().map(|(_, (_, id))| id));

    FlowColumn::el([
        match remote_packages {
            PackagesState::Loading => Text::el("Loading..."),
            PackagesState::Loaded(remote_packages) => PackageList::el(
                remote_packages
                    .into_iter()
                    .filter(|package| !loaded_package_ids.contains(&package.id))
                    .map(|package| DisplayPackage {
                        source: DisplayPackageSource::Remote {
                            url: package.url.clone(),
                        },
                        name: package.name,
                        version: package.version,
                        authors: package.authors,
                        description: package.description,
                    })
                    .collect(),
            ),
            PackagesState::Error(error) => Text::el(error),
        },
        Button::new("Load package from URL", |_| {
            PackageLoadShow.send_local(crate::packages::this::entity())
        })
        .el(),
    ])
    .with(space_between_items(), 8.0)
}

#[derive(Clone, Debug)]
struct DisplayPackage {
    source: DisplayPackageSource,
    name: String,
    version: String,
    authors: Vec<String>,
    description: Option<String>,
}

#[derive(Clone, Debug)]
enum DisplayPackageSource {
    Local { id: EntityId, enabled: bool },
    Remote { url: String },
}

#[element_component]
fn PackageList(_hooks: &mut Hooks, packages: Vec<DisplayPackage>) -> Element {
    let mut packages = packages;
    packages.sort_by_key(|package| package.name.clone());

    FlowColumn::el(packages.into_iter().map(Package::el))
        .with(space_between_items(), 8.0)
        .with(min_width(), 400.0)
}

#[element_component]
fn Package(_hooks: &mut Hooks, package: DisplayPackage) -> Element {
    fn button(text: impl Into<String>, action: impl Fn() + Send + Sync + 'static) -> Element {
        Button::new(text.into(), move |_| action())
            .style(ButtonStyle::Inline)
            .el()
    }

    with_rect(FlowRow::el([
        // Image (ideally, this would be the package icon)
        ImageFromUrl {
            url: assets::url("construction.png"),
        }
        .el()
        .with(width(), 64.0)
        .with(height(), 64.0),
        // Contents
        FlowColumn::el([
            // Header
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
            // Description
            Text::el(package.description.as_deref().unwrap_or("No description")),
            // Buttons
            match &package.source {
                DisplayPackageSource::Local { id, enabled } => {
                    let id = *id;
                    let enabled = *enabled;
                    FlowRow::el([
                        button(if enabled { "Disable" } else { "Enable" }, move || {
                            PackageSetEnabled {
                                id,
                                enabled: !enabled,
                            }
                            .send_server_reliable();
                        }),
                        button("View", move || {
                            PackageShow { id }.send_local(crate::packages::this::entity())
                        }),
                    ])
                    .with(space_between_items(), 8.0)
                }
                DisplayPackageSource::Remote { url } => {
                    let url = url.to_string();
                    button("Load", move || {
                        PackageLoad { url: url.clone() }.send_server_reliable();
                    })
                }
            },
        ])
        .with(space_between_items(), 4.0),
    ]))
    .with(space_between_items(), 8.0)
    .with(background_color(), vec4(0., 0., 0., 0.5))
    .with(fit_horizontal(), Fit::Parent)
    .with_padding_even(8.0)
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
