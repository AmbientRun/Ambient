use ambient_core::runtime;
use ambient_network::{client::game_client, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID};
use anyhow::Context;
use tokio::io::AsyncWriteExt;

use super::Bindings;
use crate::shared::{conversion::FromBindgen, wit};

impl wit::client_message::Host for Bindings {
    fn send(
        &mut self,
        target: wit::client_message::Target,
        name: String,
        mut data: Vec<u8>,
    ) -> anyhow::Result<()> {
        use wit::client_message::Target;
        let module_id = self.id.clone();
        let world = self.world_mut();

        match target {
            Target::NetworkUnreliable | Target::NetworkReliable => {
                let connection = world
                    .resource(game_client())
                    .as_ref()
                    .context("no game client")?
                    .connection
                    .clone();

                if matches!(target, Target::NetworkUnreliable) {
                    ambient_network::send_datagram(&connection, WASM_DATAGRAM_ID, move |bytes| {
                        byteorder::WriteBytesExt::write_u128::<byteorder::BigEndian>(
                            bytes,
                            module_id.0,
                        )
                        .unwrap();
                        bytes.append(&mut data)
                    })?;
                } else {
                    world.resource(runtime()).spawn(async move {
                        let mut outgoing_stream =
                            ambient_network::OutgoingStream::open_uni(&connection).await?;

                        {
                            let stream = outgoing_stream.stream.get_mut();
                            stream.write_u32(WASM_UNISTREAM_ID).await?;
                            stream.write_u128(module_id.0).await?;
                        }

                        outgoing_stream.send_bytes(data).await?;

                        anyhow::Ok(())
                    });
                }
            }
            Target::ModuleBroadcast => {
                unimplemented!();
            }
            Target::Module(id) => {
                let _id = id.from_bindgen();
                unimplemented!();
            }
        }

        Ok(())
    }
}
