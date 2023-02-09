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

    Ok(())
}
