use ambient_network::client::game_client;
use anyhow::Context;

use super::Bindings;
use crate::shared::{conversion::FromBindgen, implementation::message, wit};

impl wit::client_message::Host for Bindings {
    fn send(
        &mut self,
        target: wit::client_message::Target,
        name: String,
        data: Vec<u8>,
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

                message::send_networked(
                    world,
                    connection,
                    module_id,
                    &name,
                    &data,
                    matches!(target, Target::NetworkReliable),
                )
            }
            Target::ModuleBroadcast => message::send_local(world, module_id, None, name, data),
            Target::Module(id) => {
                message::send_local(world, module_id, Some(id.from_bindgen()), name, data)
            }
        }
    }
}
