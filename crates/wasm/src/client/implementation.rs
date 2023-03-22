use ambient_core::runtime;
use ambient_network::{client::game_client, WASM_DATAGRAM_ID, WASM_UNISTREAM_ID};
use anyhow::Context;

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
                    use byteorder::WriteBytesExt;
                    let mut payload = vec![];

                    payload.write_u128::<byteorder::BigEndian>(module_id.0)?;

                    payload.write_u32::<byteorder::BigEndian>(name.len().try_into()?)?;
                    payload.append(&mut name.into_bytes());

                    payload.append(&mut data);

                    ambient_network::send_datagram(&connection, WASM_DATAGRAM_ID, payload)?;
                } else {
                    use tokio::io::AsyncWriteExt;

                    world.resource(runtime()).spawn(async move {
                        let mut outgoing_stream =
                            ambient_network::OutgoingStream::open_uni_with_id(
                                &connection,
                                WASM_UNISTREAM_ID,
                            )
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
