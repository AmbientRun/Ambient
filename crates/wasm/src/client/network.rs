use ambient_core::async_ecs::async_run;
use ambient_ecs::World;
use ambient_network::{
    client::{
        bi_stream_handlers, datagram_handlers, uni_stream_handlers, PlatformRecvStream,
        PlatformSendStream,
    },
    log_network_result, unwrap_log_network_err, WASM_BISTREAM_ID, WASM_DATAGRAM_ID,
    WASM_UNISTREAM_ID,
};
use ambient_std::asset_cache::AssetCache;

use ambient_sys::task::PlatformBoxFuture;
use anyhow::Context;
use bytes::Bytes;

use std::{pin::Pin, sync::Arc};

use crate::shared::implementation::message::{self, process_network_message, read_unistream};

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
    _send_stream: PlatformSendStream,
    _recv_stream: PlatformRecvStream,
) -> PlatformBoxFuture<()> {
    unimplemented!("Bistreams are not supported");
}

fn on_unistream(
    world: &mut World,
    _asset_cache: AssetCache,
    mut recv_stream: PlatformRecvStream,
) -> PlatformBoxFuture<()> {
    // Reads an incoming unistream and dispatches to WASM
    let async_run = world.resource(async_run()).clone();
    PlatformBoxFuture::new(async move {
        let (remote_module_id, name, data) =
            unwrap_log_network_err!(read_unistream(Pin::new(&mut recv_stream))
                .await
                .context("Failed to read uni stream"));

        async_run.run(move |world| {
            log_network_result!(process_network_message(
                world,
                None,
                remote_module_id,
                name,
                data
            ));
        });
    })
}
