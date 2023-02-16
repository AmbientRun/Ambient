pub(crate) fn init() -> anyhow::Result<()> {
    kiwi_app::init_all_components();
    kiwi_network::init_all_components();
    kiwi_physics::init_all_components();
    kiwi_wasm::shared::init_components();
    kiwi_decals::init_components();
    kiwi_world_audio::init_components();
    kiwi_primitives::init_components();
    kiwi_project::init_components();
    kiwi_object::init_components();
    kiwi_sky::init_components();
    kiwi_water::init_components();

    crate::player::init_all_components();

    Ok(())
}

#[cfg(not(feature = "production"))]
pub(crate) mod dev;
