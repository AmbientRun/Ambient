use ambient_ecs::World;
use ambient_network::{
    log_network_result,
    server::{bi_stream_handlers, datagram_handlers, uni_stream_handlers, SharedServerState},
    WASM_BISTREAM_ID, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID,
};
use ambient_std::asset_cache::AssetCache;

use bytes::Bytes;
use quinn::{RecvStream, SendStream};

use std::sync::Arc;

use crate::shared::implementation::message;

pub fn initialize(world: &mut World) {
    world
        .resource_mut(datagram_handlers())
        .insert(WASM_DATAGRAM_ID, Arc::new(on_datagram));

    world
        .resource_mut(bi_stream_handlers())
        .insert(WASM_BISTREAM_ID, Arc::new(on_bistream));

    world
        .resource_mut(uni_stream_handlers())
        .insert(WASM_UNISTREAM_ID, Arc::new(on_unistream));
}

#[allow(clippy::ptr_arg)]
fn on_datagram(state: SharedServerState, _asset_cache: AssetCache, user_id: &String, bytes: Bytes) {
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
    _user_id: &String,
    _send_stream: SendStream,
    _recv_stream: RecvStream,
) {
    unimplemented!("Bistreams are not supported");
}

#[allow(clippy::ptr_arg)]
fn on_unistream(
    state: SharedServerState,
    _asset_cache: AssetCache,
    user_id: &String,
    recv_stream: RecvStream,
) {
    let mut state = state.lock();
    let Some(world) = state.get_player_world_mut(user_id) else {
        log::warn!("Failed to find player world for {user_id} when processing unistream");
        return;
    };

    message::on_unistream(world, Some(user_id.to_owned()), recv_stream)
}
