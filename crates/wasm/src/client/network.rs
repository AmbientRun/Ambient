use ambient_ecs::World;
use ambient_network::{
    client::{bi_stream_handlers, datagram_handlers, uni_stream_handlers},
    log_network_result, WASM_BISTREAM_ID, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID,
};
use ambient_std::asset_cache::AssetCache;
use bytes::Bytes;
use quinn::{RecvStream, SendStream};
use std::sync::Arc;

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

fn on_datagram(world: &mut World, _asset_cache: AssetCache, bytes: Bytes) {
    log_network_result!(crate::shared::network::on_datagram(world, None, bytes));
}

fn on_bistream(
    _world: &mut World,
    _asset_cache: AssetCache,
    _send_stream: SendStream,
    _recv_stream: RecvStream,
) {
    // use tokio::io::AsyncReadExt;
    unimplemented!();
}

fn on_unistream(world: &mut World, _asset_cache: AssetCache, recv_stream: RecvStream) {
    crate::shared::network::on_unistream(world, None, recv_stream)
}
