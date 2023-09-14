use ambient_api::prelude::*;
use packages::package_manager;

#[main]
fn main() {
    entity::add_component(
        package_manager::entity(),
        package_manager::components::mod_manager_for(),
        packages::this::entity(),
    );
}
