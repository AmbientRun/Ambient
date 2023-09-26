use ambient_api::{
    core::package::{
        components::{id, main_package_id},
        concepts::{Package, PackageOptional},
    },
    prelude::*,
};

mod shared;

mod server;
use server::*;

#[main]
pub async fn main() {
    let is_main_package = entity::wait_for_component(entity::resources(), main_package_id()).await
        == Some(packages::this::entity());

    if is_main_package {
        entity::add_component(
            packages::this::entity(),
            packages::this::components::mod_manager_for(),
            packages::this::entity(),
        );

        let id = entity::get_component(packages::this::entity(), id()).unwrap();

        for i in 0..4 {
            Package {
                is_package: (),
                enabled: true,
                id: format!("package{i}"),
                name: format!("Loaded Package {i}"),
                version: "0.0.1".to_string(),
                authors: vec!["Ambient".to_string()],
                asset_url: "http://not.a.valid.url".to_string(),
                client_modules: vec![],
                server_modules: vec![],
                optional: PackageOptional {
                    description: Some(format!("Description for LP{i}")),
                    repository: None,
                    for_playables: Some(vec![id.clone()]),
                },
            }
            .spawn();
        }
    }

    package_load::main();
    package_view::main();
    package_manager::main(is_main_package);
}
