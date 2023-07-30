use ambient_api::{anyhow, core::wasm::components::bytecode_from_url, prelude::*};
use serde::{de::DeserializeOwned, Deserialize};

use crate::afps::afps_ember_reloader::messages::{
    EmberLoad, EmberLoadSuccess, ErrorMessage, WasmReplaceBytecodeUrl,
};

pub fn main() {
    EmberLoad::subscribe(|source, msg| {
        let Some(user_id) = source.client_user_id() else { return; };
        let url = msg.url.strip_suffix('/').unwrap_or(&msg.url).to_owned();
        run_async(async move {
            match get_manifest_and_metadata(&url).await {
                Ok((manifest, metadata)) => {
                    let ember = &manifest.ember;
                    let make_url = |suffix: String| format!("{}/build/{}", url, suffix);

                    EmberLoadSuccess {
                        id: ember.id.to_string(),
                        name: ember.name.clone(),
                        authors: ember.authors.clone(),
                        version: ember
                            .version
                            .as_ref()
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                        client_wasms: metadata
                            .client_component_paths
                            .into_iter()
                            .map(make_url)
                            .collect(),
                        server_wasms: metadata
                            .server_component_paths
                            .into_iter()
                            .map(make_url)
                            .collect(),
                    }
                    .send_client_targeted_reliable(user_id);
                }
                Err(err) => {
                    ErrorMessage::new(err.to_string()).send_client_targeted_reliable(user_id);
                }
            };
        });
    });

    WasmReplaceBytecodeUrl::subscribe(|_, msg| {
        entity::set_component(msg.id, bytecode_from_url(), msg.url);
    });
}

async fn get_manifest_and_metadata(
    url: &str,
) -> anyhow::Result<(ambient_project::Manifest, Metadata)> {
    let manifest = get_toml(&format!("{url}/build/ambient.toml")).await?;
    let metadata = get_toml(&format!("{url}/build/metadata.toml")).await?;

    Ok((manifest, metadata))
}

async fn get_toml<T: DeserializeOwned>(url: &str) -> anyhow::Result<T> {
    let response = http::get(url).await;

    match response {
        Ok(msg) => Ok(toml::from_str::<T>(&String::from_utf8(msg)?)?),
        Err(err) => Err(anyhow!(err)),
    }
}

#[derive(Deserialize, Debug)]
struct Metadata {
    client_component_paths: Vec<String>,
    server_component_paths: Vec<String>,
}
