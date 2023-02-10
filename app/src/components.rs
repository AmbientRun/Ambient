pub(crate) fn init() -> anyhow::Result<()> {
    kiwi_app::init_all_components();
    kiwi_network::init_all_components();
    kiwi_physics::init_all_components();
    kiwi_scripting::shared::init_components();
    kiwi_decals::init_components();
    kiwi_world_audio::init_components();
    kiwi_primitives::init_components();
    kiwi_project::init_components();

    crate::player::init_all_components();

    Ok(())
}

#[cfg(not(feature = "production"))]
pub(crate) mod dev {
    use kiwi_ecs::ExternalComponentAttributes;

    pub fn build_components_toml() -> toml_edit::Document {
        let mut doc = toml_edit::Document::new();
        {
            let mut project = toml_edit::Table::new();
            project.decor_mut().set_prefix("# The following file is auto-generated. Please do not change this file.\n");
            project.insert("id", toml_edit::value("runtime_components"));
            project.insert("name", toml_edit::value("Runtime Components"));
            project.insert("version", toml_edit::value(env!("CARGO_PKG_VERSION")));
            doc.insert("project", toml_edit::Item::Table(project));
        }

        {
            let component_registry = kiwi_ecs::ComponentRegistry::get();

            let mut all_primitive = component_registry.all_primitive().collect::<Vec<_>>();
            all_primitive.sort_by_key(|pc| pc.desc.path());

            let mut components = toml_edit::Table::new();
            components.set_implicit(true);
            for component in all_primitive {
                use toml_edit::value;

                let desc = component.desc;
                let Some(name) = desc.name() else { continue };
                let Some(description) = desc.description() else { continue };

                if !description.ends_with('.') {
                    log::warn!("`{}`'s description did not end in a full stop. Is it grammatical?", component.desc.path());
                }

                let mut table = toml_edit::Table::new();
                table.insert(
                    "type",
                    match component.ty.decompose_container_type() {
                        Some((container_type, element_type)) => value(toml_edit::InlineTable::from_iter([
                            ("type", container_type.as_str()),
                            ("element_type", element_type.as_str().expect("invalid container type")),
                        ])),
                        None => value(component.ty.as_str().expect("invalid component type")),
                    },
                );
                table.insert("name", value(name));
                table.insert("description", value(description));
                {
                    let attrs = ExternalComponentAttributes::from_existing_component(desc);
                    table.insert("attributes", value(toml_edit::Array::from_iter(attrs.flags.iter())));
                }

                components.insert(&component.desc.path(), toml_edit::Item::Table(table));
            }
            doc.insert("components", toml_edit::Item::Table(components));
        }

        doc
    }
}
