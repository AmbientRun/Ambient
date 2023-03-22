use std::io::{Cursor, Read};

use ambient_core::{
    async_ecs::{async_run, AsyncRun},
    runtime,
};
use ambient_ecs::{Entity, EntityId, World};
use ambient_network::log_network_result;
use anyhow::Context;
use bytes::Bytes;
use quinn::RecvStream;

use crate::shared::{message, module_state, remote_paired_id, run, RunContext};

pub const MAX_STREAM_LENGTH: usize = 10 * 1024 * 1024;

pub fn on_datagram(world: &mut World, user_id: Option<String>, bytes: Bytes) -> anyhow::Result<()> {
    use byteorder::ReadBytesExt;

    let mut cursor = Cursor::new(&bytes);
    let remote_module_id = cursor.read_u128::<byteorder::BigEndian>()?;
    let remote_module_id = EntityId(remote_module_id);
    let Ok(module_id) = world.get(remote_module_id, remote_paired_id()) else {
        log::warn!("Failed to get remote paired ID for datagram for remote module {remote_module_id}");
        return Ok(());
    };

    let name_len = usize::try_from(cursor.read_u32::<byteorder::BigEndian>()?)?;
    let mut name = vec![0u8; name_len];
    cursor.read_exact(&mut name)?;
    let name = String::from_utf8(name)?;

    let position = cursor.position();
    let data = &bytes[usize::try_from(position)?..];

    let mut entity = Entity::new().with(message::data(), data.to_vec());

    if let Some(user_id) = user_id {
        entity.set(message::source_network_user_id(), user_id.to_owned());
    } else {
        entity.set(message::source_network(), ());
    }

    run(
        world,
        module_id,
        world.get_cloned(module_id, module_state())?,
        &RunContext::new(
            world,
            format!("{}/{}", ambient_event_types::MODULE_MESSAGE, name),
            entity,
        ),
    );

    Ok(())
}

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
            log_network_result!(async_run_handler(
                world,
                user_id,
                remote_module_id,
                name,
                data
            ));
        });

        Ok(())
    }

    fn async_run_handler(
        world: &mut World,
        user_id: Option<String>,
        remote_module_id: EntityId,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        let module_id = world.get(remote_module_id, remote_paired_id()).with_context(
            || format!("Failed to get remote paired ID for unistream for remote module {remote_module_id}")
        )?;

        let mut entity = Entity::new().with(message::data(), data.to_vec());

        if let Some(user_id) = user_id {
            entity.set(message::source_network_user_id(), user_id.to_owned());
        } else {
            entity.set(message::source_network(), ());
        }

        let state = world.get_cloned(module_id, module_state())?;
        run(
            world,
            module_id,
            state,
            &RunContext::new(
                world,
                format!("{}/{}", ambient_event_types::MODULE_MESSAGE, name),
                entity,
            ),
        );

        Ok(())
    }
}
