//! Used to implement all the server-specific host functions.
//!
//! If implementing a trait that is also available on the client, it should go in [super].

use std::{future::Future, str::FromStr};

use ambient_core::{
    asset_cache,
    async_ecs::async_run,
    player::{is_player, user_id},
    runtime,
};
use ambient_ecs::{
    generated::{messages::HttpResponse, types::HttpMethod},
    query, EntityId, World,
};
use ambient_native_std::asset_url::AbsAssetUrl;
use ambient_network::server::player_transport;
use reqwest::header::{HeaderMap, HeaderName};

use super::super::Bindings;

use crate::shared::{
    self,
    conversion::FromBindgen,
    implementation::message,
    message::{MessageExt, Target},
};

mod physics;

#[async_trait::async_trait]
impl shared::wit::server_asset::Host for Bindings {}

#[async_trait::async_trait]
impl shared::wit::server_message::Host for Bindings {
    fn send(
        &mut self,
        target: shared::wit::server_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> wasm_bridge::Result<()> {
        use shared::wit::server_message::Target as WitTarget;
        let module_id = self.id;
        let world = self.world_mut();

        match target {
            WitTarget::ClientBroadcastUnreliable => {
                send_networked(world, None, module_id, name, data, false)
            }
            WitTarget::ClientBroadcastReliable => {
                send_networked(world, None, module_id, name, data, true)
            }
            WitTarget::ClientTargetedUnreliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, false)
            }
            WitTarget::ClientTargetedReliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, true)
            }
            WitTarget::LocalBroadcast(include_self) => {
                message::send_local(world, module_id, Target::All { include_self }, name, data)
            }
            WitTarget::Local(id) => message::send_local(
                world,
                module_id,
                Target::PackageOrModule(id.from_bindgen()),
                name,
                data,
            ),
        }
    }
}

fn send_networked(
    world: &World,
    target_user_id: Option<String>,
    module_id: EntityId,
    name: String,
    data: Vec<u8>,
    reliable: bool,
) -> wasm_bridge::Result<()> {
    let connections: Vec<_> = query((user_id(), player_transport()))
        .incl(is_player())
        .iter(world, None)
        .filter(|(_, (uid, _))| {
            target_user_id
                .as_ref()
                .map(|target_uid| target_uid == *uid)
                .unwrap_or(true)
        })
        .map(|(_, (_, connection))| connection.clone())
        .collect();

    for conn in connections {
        message::send_networked(world, conn, module_id, &name, &data, reliable)?;
    }

    Ok(())
}

impl shared::wit::server_http::Host for Bindings {
    fn get(&mut self, url: String, headers: Vec<(String, String)>) -> wasm_bridge::Result<u64> {
        self.http_request_impl(HttpMethod::Get, url, headers, None)
    }

    fn post(
        &mut self,
        url: String,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
    ) -> wasm_bridge::Result<u64> {
        self.http_request_impl(HttpMethod::Post, url, headers, body)
    }
}

impl Bindings {
    fn http_request_impl(
        &mut self,
        method: HttpMethod,
        url: String,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
    ) -> wasm_bridge::Result<u64> {
        if self.hosted {
            anyhow::bail!("HTTP requests are not supported on hosted servers");
        }

        let id = self.id;
        let client = self.reqwest_client.clone();
        let response_id = self.last_http_request_id;
        self.last_http_request_id += 1;
        let world = self.world_mut();

        let assets = world.resource(asset_cache());
        let runtime = world.resource(runtime());
        let async_run = world.resource(async_run()).clone();

        let resolved_url = AbsAssetUrl::from_str(&url)?
            .to_download_url(assets)?
            .to_string();

        let request = match method {
            HttpMethod::Get => client.get(&resolved_url),
            HttpMethod::Post => client.post(&resolved_url),
        };
        let request = match body {
            Some(body) => request.body(body),
            None => request,
        };
        let request = if !headers.is_empty() {
            let mut header_map = HeaderMap::new();
            for (key, value) in headers {
                header_map.insert(HeaderName::from_str(&key)?, value.parse()?);
            }
            request.headers(header_map)
        } else {
            request
        };

        runtime.spawn(async move {
            let wasm_response = run_with_error(response_id, async move {
                let response = request.send().await?;

                let status = response.status().as_u16() as u32;
                let body = response.bytes().await?.to_vec();
                Ok(HttpResponse {
                    response_id,
                    body,
                    status,
                    error: None,
                })
            })
            .await;

            async_run.run(move |world| {
                wasm_response.send(world, Some(id)).unwrap();
            });
        });

        async fn run_with_error(
            response_id: u64,
            fut: impl Future<Output = wasm_bridge::Result<HttpResponse>>,
        ) -> HttpResponse {
            match fut.await {
                Ok(response) => response,
                Err(err) => HttpResponse {
                    response_id,
                    body: vec![],
                    status: 0,
                    error: Some(err.to_string()),
                },
            }
        }

        Ok(response_id)
    }
}

impl shared::wit::server_ambient_package::Host for Bindings {
    fn load(&mut self, url: String) -> anyhow::Result<()> {
        ambient_package_semantic_native::add(self.world_mut(), url, false)?;
        Ok(())
    }
}
