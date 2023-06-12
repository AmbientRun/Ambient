//! Used to implement all the server-specific host functions.
//!
//! If implementing a trait that is also available on the client, it should go in [super].

use ambient_core::player::{player, user_id};
use ambient_ecs::{query, EntityId, World};
use ambient_network::server::player_connection;

use super::super::Bindings;

use crate::shared::{self, conversion::FromBindgen, implementation::message, message::Target};

#[cfg(all(feature = "wit", feature = "physics"))]
mod physics;

#[cfg(feature = "wit")]
impl shared::wit::server_message::Host for Bindings {
    fn send(
        &mut self,
        target: shared::wit::server_message::Target,
        name: String,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        use shared::wit::server_message::Target as WitTarget;
        let module_id = self.id;
        let world = self.world_mut();

        match target {
            WitTarget::ClientBroadcastUnreliable => {
                send_networked(world, None, module_id, name, data, false)
            }
            WitTarget::ClientBroadcastReliable => {
                send_networked(world, None, module_id, name, data, true)
            }
            WitTarget::ClientTargetedUnreliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, false)
            }
            WitTarget::ClientTargetedReliable(user_id) => {
                send_networked(world, Some(user_id), module_id, name, data, true)
            }
            WitTarget::LocalBroadcast(include_self) => {
                message::send_local(world, module_id, Target::All { include_self }, name, data)
            }
            WitTarget::Local(id) => message::send_local(
                world,
                module_id,
                Target::Module(id.from_bindgen()),
                name,
                data,
            ),
        }
    }
}

fn send_networked(
    world: &World,
    target_user_id: Option<String>,
    module_id: EntityId,
    name: String,
    data: Vec<u8>,
    reliable: bool,
) -> anyhow::Result<()> {
    let connections: Vec<_> = query((user_id(), player_connection()))
        .incl(player())
        .iter(world, None)
        .filter(|(_, (uid, _))| {
            target_user_id
                .as_ref()
                .map(|target_uid| target_uid == *uid)
                .unwrap_or(true)
        })
        .map(|(_, (_, connection))| connection.clone())
        .collect();

    for conn in connections {
        message::send_networked(world, conn, module_id, &name, &data, reliable)?;
    }

    Ok(())
}
