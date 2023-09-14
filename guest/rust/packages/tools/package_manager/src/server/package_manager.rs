use std::collections::HashSet;

use ambient_api::{anyhow, core::package::components::enabled, prelude::*};
use ambient_package::Manifest;
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

pub fn main() {
    PackageSetEnabled::subscribe(|_, msg| {
        entity::set_component(msg.id, enabled(), msg.enabled);
    });

    PackageRemoteRequest::subscribe(|ctx, _| {
        let Some(user_id) = ctx.client_user_id() else {
            return;
        };

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
    });
}

async fn process_request() -> anyhow::Result<PackageRemoteResponse> {
    let mod_manager_for = entity::get_component(
        packages::this::entity(),
        packages::this::components::mod_manager_for(),
    )
    .and_then(|id| entity::get_component(id, ambient_api::core::package::components::id()));

    let mut api_url = "https://api.ambient.run/packages/list".to_string();
    if let Some(id) = &mod_manager_for {
        // TODO: use for_playables filtering when available
        api_url.push_str("?content_contains=Mod");
    }

    let mut api_packages: Vec<PackageListApiJson> =
        serde_json::from_slice(&http::get(&api_url).await?)?;

    let ignore_content_types: HashSet<&str> = HashSet::from_iter(["Schema", "Playable"]);
    api_packages.retain(|pkg| {
        HashSet::from_iter(pkg.content.iter().map(|s| s.as_str()))
            .is_disjoint(&ignore_content_types)
    });

    let mut packages_json = vec![];
    for api_package in api_packages {
        let url = format!(
            "https://assets.ambient.run/{}/ambient.toml",
            api_package.latest_deployment
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
