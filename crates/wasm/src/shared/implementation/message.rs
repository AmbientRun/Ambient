use ambient_core::runtime;
use ambient_ecs::{EntityId, World};
use ambient_network::{WASM_DATAGRAM_ID, WASM_UNISTREAM_ID};
use quinn::Connection;

pub fn send(
    world: &World,
    connection: Connection,
    module_id: EntityId,
    name: &str,
    data: &[u8],
    reliable: bool,
) -> anyhow::Result<()> {
    if reliable {
        send_reliable(world, connection, module_id, name, data);
        Ok(())
    } else {
        send_unreliable(connection, module_id, name, data)
    }
}

fn send_unreliable(
    connection: Connection,
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

    Ok(ambient_network::send_datagram(
        &connection,
        WASM_DATAGRAM_ID,
        payload,
    )?)
}

fn send_reliable(
    world: &World,
    connection: Connection,
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
