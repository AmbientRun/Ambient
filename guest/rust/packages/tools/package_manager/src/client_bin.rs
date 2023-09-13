use ambient_api::prelude::*;

mod shared;

mod client;
use client::{
    package_load::PackageLoad, package_manager::PackageManager, package_view::PackageViews,
};

#[main]
pub fn main() {
    App {}.el().spawn_interactive();
}

#[element_component]
pub fn App(_hooks: &mut Hooks) -> Element {
    Group::el([PackageLoad::el(), PackageManager::el(), PackageViews::el()])
}
