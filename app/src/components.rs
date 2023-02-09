use elements_ecs::ComponentRegistry;
use elements_project::Manifest;

pub(crate) fn init() -> anyhow::Result<()> {
    elements_ecs::PrimitiveComponentType::register_attributes();

    elements_app::init_all_components();
    elements_network::init_all_components();
    elements_physics::init_all_components();
    elements_scripting::shared::init_components();
    elements_decals::init_components();
    elements_world_audio::init_components();
    elements_primitives::init_components();
    elements_project::init_components();

    crate::player::init_all_components();

    // Temporary: this information should move to the ECS through attributes
    let manifest = Manifest::parse(include_str!("../elements.toml"))?;
    ComponentRegistry::get_mut().add_external_from_iterator(manifest.all_defined_components(true).map_err(anyhow::Error::msg)?.into_iter());

    Ok(())
}
