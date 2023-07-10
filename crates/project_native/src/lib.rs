use ambient_ecs::{ExternalComponentAttributes, ExternalComponentDesc, PrimitiveComponentType};

use ambient_project::{ComponentType, ItemPathBuf, Manifest};

pub fn all_defined_components(
    semantic: &ambient_project_semantic::Semantic,
) -> anyhow::Result<Vec<ExternalComponentDesc>> {
    let items = &semantic.items;
    let components = vec![];
    semantic.root_scope().visit_recursive(&items, |scope| {
        for id in scope.components.values().copied() {
            let component = items.get(id)?;

            components.push(ExternalComponentDesc {
                path: items.fully_qualified_display_path_ambient_style(&*component)?,
                ty: component_type_to_primitive(&component.type_)?,
                name: component.name.clone(),
                description: component.description.clone(),
                attributes: ExternalComponentAttributes::from_iter(
                    component.attributes.iter().map(|s| s.as_str()),
                ),
            });
        }
        Ok(())
    });
    Ok(components)
}
