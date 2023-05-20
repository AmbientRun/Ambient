use ambient_ecs::World;
use ambient_network::{
    client::{bi_stream_handlers, datagram_handlers, uni_stream_handlers, DynRecv, DynSend},
    log_network_result, WASM_BISTREAM_ID, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID,
};
use ambient_std::asset_cache::AssetCache;

use bytes::Bytes;

use std::sync::Arc;

use crate::shared::implementation::message;

pub fn initialize(world: &mut World) {
    world.resource_mut(datagram_handlers()).insert(
        WASM_DATAGRAM_ID,
        ("client_wasm_datagram", Arc::new(on_datagram)),
    );

    world.resource_mut(bi_stream_handlers()).insert(
        WASM_BISTREAM_ID,
        ("client_wasm_bi_stream", Arc::new(on_bistream)),
    );

    world.resource_mut(uni_stream_handlers()).insert(
        WASM_UNISTREAM_ID,
        ("client_wasm_uni_stream", Arc::new(on_unistream)),
    );
}

fn on_datagram(world: &mut World, _asset_cache: AssetCache, bytes: Bytes) {
    log_network_result!(message::on_datagram(world, None, bytes));
}

fn on_bistream(
    _world: &mut World,
    _asset_cache: AssetCache,
    _send_stream: DynSend,
    _recv_stream: DynRecv,
) {
    unimplemented!("Bistreams are not supported");
}

fn on_unistream(world: &mut World, _asset_cache: AssetCache, recv_stream: DynRecv) {
    message::on_unistream(world, None, recv_stream)
}
