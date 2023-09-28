use ambient_api::{
    core::{messages::ModuleUnload, package::components::enabled},
    prelude::*,
};
use packages::level_cubicide::{self, components::include_corners};

#[main]
pub async fn main() {
    entity::add_component(level_cubicide::entity(), include_corners(), false);
    entity::set_component(level_cubicide::entity(), enabled(), true);

    ModuleUnload::subscribe(|_| {
        entity::add_component(level_cubicide::entity(), include_corners(), true);
        entity::set_component(level_cubicide::entity(), enabled(), true);
    });
}
