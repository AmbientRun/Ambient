use crate::{
    packages::{
        self,
        this::messages::{
            PackageLoad, PackageLoadShow, PackageRemoteRequest, PackageRemoteResponse,
            PackageSetEnabled,
        },
    },
    shared::PackageJson,
};
use ambient_api::{
    core::package::{
        components::{description, for_playables, id, is_package},
        concepts::Package as PackageConcept,
    },
    element::{
        use_effect, use_entity_component, use_module_message, use_query, use_spawn, use_state,
    },
    prelude::*,
};
use ambient_brand_theme::design_tokens::{
    BRANDLIGHT::{SEMANTIC_MAIN_ELEMENTS_INACTIVE, SEMANTIC_MAIN_ELEMENTS_SECONDARY},
    LIGHT::SEMANTIC_MAIN_ELEMENTS_PRIMARY,
};
use ambient_brand_theme::{
    window_style, AmbientInternalStyle, Toggle, SEMANTIC_MAIN_ELEMENTS_TERTIARY,
};
use std::{collections::HashSet, fmt};

use super::use_hotkey_toggle;

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
    }
    .to_uppercase();

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
        child: if mod_manager_for.is_some() {
            ModManagerInner::el()
        } else {
            PackageManagerInner::el()
        }
        .with(space_between_items(), 4.0)
        .with_margin_even(8.),
    }
    .el()
}

#[element_component]
fn ModManagerInner(_hooks: &mut Hooks) -> Element {
    FlowColumn::el([PackagesRemote::el(false)])
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
        .with_tab(ListTab::Remote, || {
            FlowColumn::el(vec![
                PackagesRemote::el(true),
                Button::new("Load package from URL", |_| {
                    PackageLoadShow.send_local(crate::packages::this::entity())
                })
                .el(),
            ])
        })
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
                // version: package.version,
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
fn PackagesRemote(hooks: &mut Hooks, filter_away_local: bool) -> Element {
    let remote_packages = use_remote_packages(hooks);
    let loaded_packages = use_query(hooks, (is_package(), id()));

    let loaded_package_ids: HashSet<String> =
        HashSet::from_iter(loaded_packages.into_iter().map(|(_, (_, id))| id));

    FlowColumn::el([match remote_packages {
        PackagesState::Loading => Text::el("Loading..."),
        PackagesState::Loaded(remote_packages) => PackageList::el(
            remote_packages
                .into_iter()
                .filter(|package| {
                    if filter_away_local {
                        !loaded_package_ids.contains(&package.id)
                    } else {
                        true
                    }
                })
                .map(|package| DisplayPackage {
                    source: DisplayPackageSource::Remote {
                        url: package.url.clone(),
                    },
                    name: package.name,
                    // version: package.version,
                    authors: package.authors,
                    description: package.description,
                })
                .collect(),
        ),
        PackagesState::Error(error) => Text::el(error),
    }])
    .with(space_between_items(), 8.0)
}

#[derive(Clone, Debug)]
struct DisplayPackage {
    source: DisplayPackageSource,
    name: String,
    // version: String,
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

    let sep = Rectangle::el()
        .hex_background(SEMANTIC_MAIN_ELEMENTS_INACTIVE)
        .with(height(), 1.)
        .with(fit_horizontal(), Fit::Parent);

    FlowColumn::el(itertools::intersperse(
        packages.into_iter().map(Package::el),
        sep,
    ))
    .with(space_between_items(), 0.0)
}

#[element_component]
fn Package(_hooks: &mut Hooks, package: DisplayPackage) -> Element {
    let enabled = match &package.source {
        DisplayPackageSource::Local { enabled, .. } => *enabled,
        _ => false,
    };

    FlowRow::el([
        // Header
        FlowColumn::el([
            Text::el(package.name.to_uppercase())
                .mono_s_500upp()
                .hex_text_color(SEMANTIC_MAIN_ELEMENTS_PRIMARY),
            Text::el(if package.authors.is_empty() {
                "No authors specified".to_string().to_uppercase()
            } else {
                package.authors.join(", ").to_uppercase()
            })
            .mono_s_500upp()
            .hex_text_color(SEMANTIC_MAIN_ELEMENTS_TERTIARY),
            // Description
            if let Some(desc) = package.description {
                Text::el(desc)
                    .body_s_500()
                    .hex_text_color(SEMANTIC_MAIN_ELEMENTS_SECONDARY)
            } else {
                Element::new()
            },
        ])
        .with(space_between_items(), 4.0)
        .with(width(), 250.)
        .with(fit_horizontal(), Fit::None),
        // Buttons
        Toggle::el(
            enabled,
            cb(move |_| match &package.source {
                DisplayPackageSource::Local { id, enabled } => {
                    let id = *id;
                    let enabled = *enabled;
                    PackageSetEnabled {
                        id,
                        enabled: !enabled,
                    }
                    .send_server_reliable()
                }
                DisplayPackageSource::Remote { url } => {
                    let url = url.to_string();
                    PackageLoad {
                        url: url.clone(),
                        enabled: true,
                    }
                    .send_server_reliable();
                }
            }),
        ),
    ])
    .with(space_between_items(), 4.0)
    .with(fit_horizontal(), Fit::Children)
    .with(padding(), vec4(12., 4., 12., 4.))
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
