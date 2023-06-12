use ambient_core::{async_ecs::async_run, runtime};
use ambient_ecs::World;
use ambient_network::{
    log_network_result,
    server::{bi_stream_handlers, datagram_handlers, uni_stream_handlers, SharedServerState},
    unwrap_log_network_err, DynRecv, DynSend, WASM_BISTREAM_ID, WASM_DATAGRAM_ID,
    WASM_UNISTREAM_ID,
};
use ambient_std::asset_cache::AssetCache;

use anyhow::Context;
use bytes::Bytes;

use std::sync::Arc;

use crate::shared::implementation::message::{self, process_network_message, read_unistream};

pub fn initialize(world: &mut World) {
    world.resource_mut(datagram_handlers()).insert(
        WASM_DATAGRAM_ID,
        ("server_wasm_datagram", Arc::new(on_datagram)),
    );

    world.resource_mut(bi_stream_handlers()).insert(
        WASM_BISTREAM_ID,
        ("server_wasm_bi_stream", Arc::new(on_bistream)),
    );

    world.resource_mut(uni_stream_handlers()).insert(
        WASM_UNISTREAM_ID,
        ("server_wasm_uni_stream", Arc::new(on_unistream)),
    );
}

#[allow(clippy::ptr_arg)]
fn on_datagram(state: SharedServerState, _asset_cache: AssetCache, user_id: &str, bytes: Bytes) {
    let mut state = state.lock();
    let Some(world) = state.get_player_world_mut(user_id) else {
        log::warn!("Failed to find player world for {user_id} when processing datagram");
        return;
    };

    log_network_result!(message::on_datagram(world, Some(user_id.to_owned()), bytes));
}

#[allow(clippy::ptr_arg)]
fn on_bistream(
    _state: SharedServerState,
    _asset_cache: AssetCache,
    _user_id: &str,
    _send_stream: DynSend,
    _recv_stream: DynRecv,
) {
    unimplemented!("Bistreams are not supported");
}

#[allow(clippy::ptr_arg)]
fn on_unistream(
    state: SharedServerState,
    _asset_cache: AssetCache,
    user_id: &str,
    mut recv_stream: DynRecv,
) {
    let mut state = state.lock();
    let Some(world) = state.get_player_world_mut(user_id) else {
        log::warn!("Failed to find player world for {user_id} when processing unistream");
        return;
    };

    // Reads an incoming unistream and dispatches to WASM
    let async_run = world.resource(async_run()).clone();
    let user_id = user_id.to_owned();
    world.resource(runtime()).spawn(async move {
        let (remote_module_id, name, data) =
            unwrap_log_network_err!(read_unistream(recv_stream.as_mut())
                .await
                .context("Failed to read uni stream"));

        async_run.run(move |world| {
            log_network_result!(process_network_message(
                world,
                Some(user_id),
                remote_module_id,
                name,
                data
            ));
        });
    });
}
