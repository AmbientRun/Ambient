use ambient_core::{
    async_ecs::{async_run, AsyncRun},
    runtime,
};
use ambient_ecs::{EntityId, World};
use ambient_network::{
    client::{ClientConnection, DynRecv},
    connection::Connection,
    log_network_result, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID,
};

use anyhow::Context;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use quinn::RecvStream;

use std::{
    collections::HashSet,
    io::{Cursor, Read},
    sync::Arc,
};

use crate::shared::remote_paired_id;

pub const MAX_STREAM_LENGTH: usize = 10 * 1024 * 1024;

pub fn subscribe(subscribed_events: &mut HashSet<String>, name: String) -> anyhow::Result<()> {
    subscribed_events.insert(name);
    Ok(())
}

/// Reads an incoming datagram and dispatches to WASM
pub fn on_datagram(world: &mut World, user_id: Option<String>, bytes: Bytes) -> anyhow::Result<()> {
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(&bytes);
    let remote_module_id = cursor.read_u128::<byteorder::BigEndian>()?;
    let remote_module_id = EntityId(remote_module_id);

    let name_len: usize = cursor.get_u32().try_into()?;
    let mut name = vec![0u8; name_len];
    cursor.read_exact(&mut name)?;
    let name = String::from_utf8(name)?;

    let position = cursor.position();
    let data = &bytes[usize::try_from(position)?..];

    process_network_message(world, user_id, remote_module_id, name, data.to_vec())?;

    Ok(())
}

/// Reads an incoming unistream and dispatches to WASM
pub fn on_unistream(world: &mut World, user_id: Option<String>, recv_stream: DynRecv) {
    let async_run = world.resource(async_run()).clone();
    world.resource(runtime()).spawn(async move {
        log_network_result!(unistream_handler(async_run, user_id, recv_stream).await);
    });

    async fn unistream_handler(
        async_run: AsyncRun,
        user_id: Option<String>,
        mut recv_stream: DynRecv,
    ) -> anyhow::Result<()> {
        use tokio::io::AsyncReadExt;

        let remote_module_id = recv_stream.read_u128().await?;
        let remote_module_id = EntityId(remote_module_id);

        let name_len = usize::try_from(recv_stream.read_u32().await?)?;
        let mut name = vec![0u8; name_len];
        recv_stream.read_exact(&mut name).await?;
        let name = String::from_utf8(name)?;

        let mut data = Vec::new();

        recv_stream
            .take(MAX_STREAM_LENGTH as _)
            .read_to_end(&mut data)
            .await?;

        async_run.run(move |world| {
            log_network_result!(process_network_message(
                world,
                user_id,
                remote_module_id,
                name,
                data
            ));
        });

        Ok(())
    }
}

pub fn process_network_message(
    world: &mut World,
    user_id: Option<String>,
    remote_module_id: EntityId,
    name: String,
    data: Vec<u8>,
) -> anyhow::Result<()> {
    use crate::shared::message;

    let module_id = world
        .get(remote_module_id, remote_paired_id())
        .with_context(|| {
            format!(
                "Failed to get remote paired ID for unistream for remote module {remote_module_id}"
            )
        })?;

    message::send(
        world,
        Some(module_id),
        match user_id {
            Some(user_id) => message::Source::Client(user_id),
            None => message::Source::Server,
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
    module_id: Option<EntityId>,
    name: String,
    data: Vec<u8>,
) -> anyhow::Result<()> {
    use crate::shared::message;

    message::send(
        world,
        module_id,
        message::Source::Local(source_module_id),
        name,
        data,
    );

    Ok(())
}

/// Sends a message over the network for the specified module
pub fn send_networked(
    world: &World,
    connection: Arc<dyn ClientConnection>,
    module_id: EntityId,
    name: &str,
    data: &[u8],
    reliable: bool,
) -> anyhow::Result<()> {
    if reliable {
        send_unistream(world, connection, module_id, name, data);
        Ok(())
    } else {
        send_datagram(world, &*connection, module_id, name, data)
    }
}

fn send_datagram(
    world: &World,
    connection: &dyn ClientConnection,
    module_id: EntityId,
    name: &str,
    data: &[u8],
) -> anyhow::Result<()> {
    let mut payload = BytesMut::new();

    payload.put_u128(module_id.0);

    payload.put_u32(name.len().try_into()?);
    payload.extend_from_slice(name.as_bytes());

    payload.extend_from_slice(data);

    connection.send_datagram(WASM_DATAGRAM_ID, payload.freeze())?;

    Ok(())
}

fn send_unistream(
    world: &World,
    connection: Arc<dyn ClientConnection>,
    module_id: EntityId,
    name: &str,
    data: &[u8],
) {
    let name = name.to_owned();
    let data = data.to_owned();

    world.resource(runtime()).spawn(async move {
        let mut payload = BytesMut::new();
        payload.put_u128(module_id.0);

        payload.put_u32(name.len().try_into()?);
        payload.put(name.as_bytes());

        payload.put(&data[..]);

        connection
            .request_uni(WASM_UNISTREAM_ID, payload.freeze())
            .await?;

        anyhow::Ok(())
    });
}
