use ambient_core::{
    async_ecs::{async_run, AsyncRun},
    runtime,
};
use ambient_ecs::{EntityId, World};
use ambient_network::{log_network_result, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID, connection::Connection};

use anyhow::Context;
use bytes::Bytes;
use quinn::RecvStream;

use std::io::{Cursor, Read};

use crate::shared::remote_paired_id;

pub const MAX_STREAM_LENGTH: usize = 10 * 1024 * 1024;

/// Reads an incoming datagram and dispatches to WASM
pub fn on_datagram(world: &mut World, user_id: Option<String>, bytes: Bytes) -> anyhow::Result<()> {
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(&bytes);
    let remote_module_id = cursor.read_u128::<byteorder::BigEndian>()?;
    let remote_module_id = EntityId(remote_module_id);

    let name_len = usize::try_from(cursor.read_u32::<byteorder::BigEndian>()?)?;
    let mut name = vec![0u8; name_len];
    cursor.read_exact(&mut name)?;
    let name = String::from_utf8(name)?;

    let position = cursor.position();
    let data = &bytes[usize::try_from(position)?..];

    process_network_message(world, user_id, remote_module_id, name, data.to_vec())?;

    Ok(())
}

/// Reads an incoming unistream and dispatches to WASM
pub fn on_unistream(world: &mut World, user_id: Option<String>, recv_stream: RecvStream) {
    let async_run = world.resource(async_run()).clone();
    world.resource(runtime()).spawn(async move {
        log_network_result!(unistream_handler(async_run, user_id, recv_stream).await);
    });

    async fn unistream_handler(
        async_run: AsyncRun,
        user_id: Option<String>,
        mut recv_stream: RecvStream,
    ) -> anyhow::Result<()> {
        use tokio::io::AsyncReadExt;

        let remote_module_id = recv_stream.read_u128().await?;
        let remote_module_id = EntityId(remote_module_id);

        let name_len = usize::try_from(recv_stream.read_u32().await?)?;
        let mut name = vec![0u8; name_len];
        recv_stream.read_exact(&mut name).await?;
        let name = String::from_utf8(name)?;

        let data = recv_stream.read_to_end(MAX_STREAM_LENGTH).await?;

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
            Some(user_id) => message::Source::NetworkUserId(user_id),
            None => message::Source::Network,
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
        message::Source::Module(source_module_id),
        name,
        data,
    );

    Ok(())
}

/// Sends a message over the network for the specified module
pub fn send_networked<C: Connection + 'static>(
    world: &World,
    connection: C,
    module_id: EntityId,
    name: &str,
    data: &[u8],
    reliable: bool,
) -> anyhow::Result<()> {
    if reliable {
        send_unistream(world, connection, module_id, name, data);
        Ok(())
    } else {
        send_datagram(world, connection, module_id, name, data)
    }
}

fn send_datagram<C: Connection + 'static>(
    world: &World,
    connection: C,
    module_id: EntityId,
    name: &str,
    data: &[u8],
) -> anyhow::Result<()> {
    use byteorder::WriteBytesExt;
    let mut payload = vec![];

    payload.write_u128::<byteorder::BigEndian>(module_id.0)?;

    payload.write_u32::<byteorder::BigEndian>(name.len().try_into()?)?;
    payload.extend_from_slice(name.as_bytes());

    payload.extend_from_slice(data);

    world.resource(runtime()).spawn(async move {
        ambient_network::send_datagram(
            &connection,
            WASM_DATAGRAM_ID,
            payload,
        ).await?;

        anyhow::Ok(())
    });

    Ok(())
}

fn send_unistream<C: Connection + 'static>(
    world: &World,
    connection: C,
    module_id: EntityId,
    name: &str,
    data: &[u8],
) {
    use tokio::io::AsyncWriteExt;

    let name = name.to_owned();
    let data = data.to_owned();

    world.resource(runtime()).spawn(async move {
        let mut outgoing_stream =
            ambient_network::OutgoingStream::open_uni_with_id(&connection, WASM_UNISTREAM_ID)
                .await?;

        {
            let stream = outgoing_stream.stream.get_mut();
            stream.write_u128(module_id.0).await?;

            stream.write_u32(name.len().try_into()?).await?;
            stream.write_all(name.as_bytes()).await?;

            stream.write_all(&data).await?;
        }

        anyhow::Ok(())
    });
}
