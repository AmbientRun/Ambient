use ambient_api::{anyhow, core::package::components::enabled, prelude::*};
use ambient_package::Manifest;
use ambient_shared_types::urls::{PackageContent, PackageListParams};
use serde::{Deserialize, Serialize};

use crate::{
    packages::{
        self,
        this::messages::{PackageRemoteRequest, PackageRemoteResponse, PackageSetEnabled},
    },
    shared::PackageJson,
};

#[derive(Serialize, Deserialize, Debug)]
struct PackageListApiJson {
    name: String,
    latest_deployment: String,
    content: Vec<String>,
}

pub fn main(send_dummy_data: bool) {
    PackageSetEnabled::subscribe(|_, msg| {
        entity::set_component(msg.id, enabled(), msg.enabled);
    });

    PackageRemoteRequest::subscribe(move |ctx, _| {
        let Some(user_id) = ctx.client_user_id() else {
            return;
        };

        if send_dummy_data {
            PackageRemoteResponse {
                packages: (0..4)
                    .map(|p| PackageJson {
                        url: format!("http://not.a.valid.url/{p}"),
                        name: format!("Unloaded Package {p}"),
                        id: format!("unloadedpackage{p}"),
                        version: "0.0.1".to_string(),
                        authors: vec!["Ambient".to_string()],
                        description: Some(format!("Description for UP{p}")),
                    })
                    .map(|p| serde_json::to_string(&p).unwrap())
                    .collect(),
                error: None,
            }
            .send_client_targeted_reliable(user_id)
        } else {
            run_async(async move {
                match process_request().await {
                    Ok(msg) => msg,
                    Err(err) => PackageRemoteResponse {
                        packages: vec![],
                        error: Some(err.to_string()),
                    },
                }
                .send_client_targeted_reliable(user_id)
            });
        }
    });
}

async fn process_request() -> anyhow::Result<PackageRemoteResponse> {
    let mod_manager_for = entity::get_component(
        packages::this::entity(),
        packages::this::components::mod_manager_for(),
    )
    .and_then(|id| entity::get_component(id, ambient_api::core::package::components::id()));

    let list_params = if let Some(id) = &mod_manager_for {
        PackageListParams {
            content_contains: &[PackageContent::Mod],
            for_playable: Some(id),
            ..default()
        }
    } else {
        PackageListParams {
            content_contains: &[
                PackageContent::Mod,
                PackageContent::Asset,
                PackageContent::Tool,
            ],
            ..default()
        }
    };
    let api_url = ambient_shared_types::urls::package_list_url(list_params);

    let api_packages =
        serde_json::from_slice::<Vec<PackageListApiJson>>(&http::get(&api_url).await?)?;

    let mut packages_json = vec![];
    for api_package in api_packages {
        let url = format!(
            "{}/ambient.toml",
            ambient_shared_types::urls::deployment_url(&api_package.latest_deployment)
        );

        let manifest: Manifest = toml::from_str(std::str::from_utf8(&http::get(&url).await?)?)?;

        if let Some(id) = &mod_manager_for {
            let ambient_package::PackageContent::Mod { for_playables } = manifest.package.content
            else {
                continue;
            };

            if !for_playables.contains(id) {
                continue;
            }
        }

        packages_json.push(serde_json::to_string(&PackageJson {
            url,
            name: manifest.package.name,
            id: manifest.package.id.to_string(),
            version: manifest.package.version.to_string(),
            authors: manifest.package.authors,
            description: manifest.package.description,
        })?);
    }

    Ok(PackageRemoteResponse {
        packages: packages_json,
        error: None,
    })
}
