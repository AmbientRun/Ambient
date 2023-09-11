use ambient_core::runtime;
use ambient_ecs::{generated::wasm::components::package_ref, EntityId, World};
use ambient_network::{
    client::NetworkTransport, log_network_result, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID,
};

use anyhow::Context;
use bytes::{Buf, BufMut, Bytes, BytesMut};

use std::{
    collections::HashSet,
    io::{Cursor, Read},
    pin::Pin,
    sync::Arc,
};

use crate::shared::message::Target;

pub const MAX_STREAM_LENGTH: usize = 10 * 1024 * 1024;

pub fn subscribe(subscribed_events: &mut HashSet<String>, name: String) -> anyhow::Result<()> {
    subscribed_events.insert(name);
    Ok(())
}

/// Reads an incoming datagram and dispatches to WASM
pub fn on_datagram(world: &mut World, user_id: Option<String>, bytes: Bytes) -> anyhow::Result<()> {
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(&bytes);
    let package_id = cursor.read_u128::<byteorder::BigEndian>()?;
    let package_id = EntityId(package_id);

    let name_len: usize = cursor.get_u32().try_into()?;
    let mut name = vec![0u8; name_len];
    cursor.read_exact(&mut name)?;
    let name = String::from_utf8(name)?;

    let position = cursor.position();
    let data = &bytes[usize::try_from(position)?..];

    process_network_message(world, user_id, package_id, name, data.to_vec())?;

    Ok(())
}

pub async fn read_unistream<R: ?Sized + tokio::io::AsyncRead>(
    mut recv_stream: Pin<&mut R>,
) -> anyhow::Result<(EntityId, String, Vec<u8>)> {
    use tokio::io::AsyncReadExt;

    let package_id = recv_stream.read_u128().await?;
    let package_id = EntityId(package_id);

    let name_len: usize = recv_stream
        .read_u32()
        .await?
        .try_into()
        .context("Failed to context name length")?;

    let mut name = vec![0u8; name_len];
    recv_stream.read_exact(&mut name).await?;
    let name = String::from_utf8(name)?;

    let mut data = Vec::new();
    recv_stream
        .take(MAX_STREAM_LENGTH as _)
        .read_to_end(&mut data)
        .await?;

    Ok((package_id, name, data))
}

pub fn process_network_message(
    world: &mut World,
    user_id: Option<String>,
    package_id: EntityId,
    name: String,
    data: Vec<u8>,
) -> anyhow::Result<()> {
    use crate::shared::message;

    message::send(
        world,
        Target::PackageOrModule(package_id),
        match user_id {
            Some(user_id) => message::WorldEventSource::Client(user_id),
            None => message::WorldEventSource::Server,
        },
        name,
        data,
    );

    Ok(())
}

/// Sends a message to another module on this side
pub fn send_local(
    world: &mut World,
    source_module_id: EntityId,
    target: Target,
    name: String,
    data: Vec<u8>,
) -> anyhow::Result<()> {
    use crate::shared::message;

    message::send(
        world,
        target,
        message::WorldEventSource::Local(source_module_id),
        name,
        data,
    );

    Ok(())
}

/// Sends a message over the network for the specified module
pub fn send_networked(
    world: &World,
    transport: Arc<dyn NetworkTransport>,
    module_id: EntityId,
    name: &str,
    data: &[u8],
    reliable: bool,
) -> anyhow::Result<()> {
    let package_id = world.get(module_id, package_ref())?;

    if reliable {
        send_unistream(world, transport, package_id, name, data);
        Ok(())
    } else {
        send_datagram(world, transport, package_id, name, data)
    }
}

fn send_datagram(
    world: &World,
    transport: Arc<dyn NetworkTransport>,
    package_id: EntityId,
    name: &str,
    data: &[u8],
) -> anyhow::Result<()> {
    let mut payload = BytesMut::new();

    payload.put_u128(package_id.0);

    payload.put_u32(name.len().try_into()?);
    payload.extend_from_slice(name.as_bytes());

    payload.extend_from_slice(data);

    world.resource(runtime()).spawn(async move {
        log_network_result!(
            transport
                .send_datagram(WASM_DATAGRAM_ID, payload.freeze())
                .await
        );
    });

    Ok(())
}

fn send_unistream(
    world: &World,
    transport: Arc<dyn NetworkTransport>,
    package_id: EntityId,
    name: &str,
    data: &[u8],
) {
    let name = name.to_owned();
    let data = data.to_owned();

    world.resource(runtime()).spawn(async move {
        let mut payload = BytesMut::new();
        payload.put_u128(package_id.0);

        payload.put_u32(name.len().try_into()?);
        payload.put(name.as_bytes());

        payload.put(&data[..]);

        transport
            .request_uni(WASM_UNISTREAM_ID, payload.freeze())
            .await?;

        anyhow::Ok(())
    });
}
