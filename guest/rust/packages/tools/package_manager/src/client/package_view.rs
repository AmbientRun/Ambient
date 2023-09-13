use std::collections::HashSet;

use ambient_api::{
    core::{
        package::{
            components::{is_package, name},
            concepts::Package,
        },
        text::{components::font_style, types::FontStyle},
        wasm::{components::is_module_on_server, concepts::Module},
    },
    element::{
        use_entity_concept, use_module_message, use_query, use_ref_with, use_rerender_signal,
    },
    prelude::*,
};

use crate::packages::this::messages::{PackageSetEnabled, PackageShow, WasmReload, WasmSetEnabled};

#[element_component]
pub fn PackageViews(hooks: &mut Hooks) -> Element {
    let visible_packages = use_ref_with(hooks, |_| HashSet::<EntityId>::new());
    let packages = use_query(hooks, (is_package(), name()));
    let rerender = use_rerender_signal(hooks);

    use_module_message::<PackageShow>(hooks, {
        to_owned!(rerender, visible_packages);
        move |_, _, msg| {
            visible_packages.lock().insert(msg.id);
            rerender();
        }
    });

    Group::el(packages.into_iter().map(|(package, (_, name))| {
        let package_visible = visible_packages.lock().contains(&package);

        to_owned!(rerender, visible_packages);
        Window::el(
            name,
            package_visible,
            Some(cb(move || {
                visible_packages.lock().remove(&package);
                rerender();
            })),
            PackageViewInner::el(package),
        )
    }))
}

#[element_component]
fn PackageViewInner(hooks: &mut Hooks, package_id: EntityId) -> Element {
    let package = use_entity_concept::<Package>(hooks, package_id);
    let mut modules: Vec<_> = use_query(hooks, Module::as_query())
        .into_iter()
        .filter(|(_, m)| m.package_ref == package_id)
        .map(|(id, mut m)| {
            // TODO: Figure out if we can get the optional components in the query as well,
            // this is not great
            m.optional.is_module_on_server = entity::get_component(id, is_module_on_server());
            (id, m)
        })
        .collect();

    let Some(package) = package else {
        return Text::el(format!(
            "Entity {package_id} does not exist or is not a valid package"
        ));
    };

    modules.sort_by_key(|(_, m)| {
        (
            m.optional.is_module_on_server.is_some(),
            m.module_name.clone(),
        )
    });

    let (server_modules, client_modules): (Vec<_>, Vec<_>) = modules
        .into_iter()
        .partition(|(_, m)| m.optional.is_module_on_server.is_some());

    fn render_modules(heading: String, modules: Vec<(EntityId, Module)>) -> Element {
        FlowColumn::el(Iterator::chain(
            [Text::el(heading).with(font_style(), FontStyle::Bold)].into_iter(),
            modules.into_iter().map(|(id, module)| {
                FlowRow::el([
                    Checkbox::new(module.module_enabled, move |value| {
                        WasmSetEnabled::new(id, value).send_server_reliable();
                    })
                    .el(),
                    Button::new(FontAwesomeIcon::el(0xf2f1, true), move |_| {
                        WasmReload::new(id).send_server_reliable();
                    })
                    .style(ButtonStyle::Flat)
                    .el(),
                    FlowColumn::el([
                        Text::el(module.module_name),
                        Text::el(module.bytecode_from_url)
                            .small_style()
                            .with(font_style(), FontStyle::Italic),
                    ]),
                ])
                .with(space_between_items(), 4.0)
            }),
        ))
        .with(space_between_items(), 4.0)
    }

    FlowColumn::el([
        FlowRow::el([
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
        FlowRow::el([
            Checkbox::new(package.enabled, move |value| {
                PackageSetEnabled::new(package_id, value).send_server_reliable();
            })
            .el(),
            Text::el("Enabled"),
        ]),
        Text::el(
            package
                .optional
                .description
                .as_deref()
                .unwrap_or("No description"),
        ),
        render_modules("Server modules".to_string(), server_modules),
        render_modules("Client modules".to_string(), client_modules),
    ])
    .with_margin_even(STREET)
    .with(space_between_items(), STREET)
}
