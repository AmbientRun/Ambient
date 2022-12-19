use std::{any::type_name, io, sync::Arc};

use anyhow::Context;
use dashmap::DashMap;
use elements_ecs::{components, query, EntityId, World};
use elements_std::Cb;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::{client_game_state::ClientGameState, server::player_event_stream};

components!("network", {
    event_registry: Arc<ServerEventRegistry>,
});

pub trait ServerEventHandler<T>: Send + Sync + 'static {
    fn run(&self, world: &mut World, event: T) -> anyhow::Result<()>;
}

impl<T, F> ServerEventHandler<T> for F
where
    F: Fn(&mut World, T) -> anyhow::Result<()> + Send + Sync + 'static,
{
    fn run(&self, world: &mut World, event: T) -> anyhow::Result<()> {
        (self)(world, event)
    }
}

#[allow(clippy::type_complexity)]
#[derive(Debug, Clone)]
/// Does the type erasure
struct Handler {
    func: Cb<dyn for<'x> Fn(&'x mut World, &'x [u8]) -> anyhow::Result<()> + Send + Sync>,
}

impl Handler {
    pub fn new<T, F>(handler: F) -> Self
    where
        T: for<'x> Deserialize<'x>,
        F: ServerEventHandler<T>,
        F: 'static,
    {
        let func = move |w: &mut World, buf: &[u8]| -> anyhow::Result<()> {
            let buf: &&[u8] = &buf;
            let event: T = bincode::deserialize(buf).context(format!("Failed to decode event: {}", type_name::<T>()))?;

            handler.run(w, event)?;
            Ok(())
        };

        Self { func: Cb(Arc::new(func)) }
    }
    pub fn run(&self, gs: &Mutex<ClientGameState>, event: Box<[u8]>) -> anyhow::Result<()> {
        (self.func)(&mut gs.lock().world, &event)
    }
}

#[derive(Default, Clone, Debug)]
pub struct ServerEventRegistry {
    handlers: DashMap<String, Handler>,
}

impl ServerEventRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a callback for an event by name.
    /// Overrides previous handler.
    ///
    /// A callback can either be a `fn` or a custom type which implements
    /// [`self::EventHandler`].
    pub fn register<T, F>(&self, func: F) -> &Self
    where
        T: for<'x> Deserialize<'x>,
        F: ServerEventHandler<T>,
        F: 'static,
    {
        let handler = Handler::new(func);
        self.handlers.insert(type_name::<T>().to_string(), handler);
        self
    }

    pub fn serialize<W, T>(writer: &mut W, event: T) -> bincode::Result<()>
    where
        W: io::Write,
        T: Serialize,
    {
        writeln!(writer, "{}", type_name::<T>())?;
        bincode::serialize_into(writer, &event)?;

        Ok(())
    }

    /// Handle an event of any type from the incoming stream.
    /// Requires the locked game state as std::MutexGuard is non-send.
    pub fn handle_event(&self, gs: &Mutex<ClientGameState>, event_name: &str, event_data: Box<[u8]>) -> anyhow::Result<()> {
        let handler = self.handlers.get(event_name);
        if let Some(handler) = handler {
            handler.run(gs, event_data).context(format!("Failed to run event handler for {event_name:?}"))?;
        } else {
            return Err(anyhow::anyhow!(format!("No handler for {event_name:?}")));
        };

        Ok(())
    }
}

/// Sends an event to a specific player.
/// An event can be of any type.
pub fn send_event<T: Serialize>(world: &World, player_id: EntityId, event: T) {
    let mut buf = Vec::new();
    ServerEventRegistry::serialize(&mut buf, event).expect("Failed to serialize event");
    if let Ok(tx) = world.get_ref(player_id, player_event_stream()) {
        if tx.send(buf.clone()).is_err() {
            log::warn!("Attempt to broadcast to disconnected player ")
        }
    }
}

/// Broadcasts an event to all connected players.
/// An event can be of any type.
pub fn broadcast_event<T: Serialize>(world: &World, event: T) {
    let mut buf = Vec::new();
    ServerEventRegistry::serialize(&mut buf, event).expect("Failed to serialize event");
    for (_, tx) in query(player_event_stream()).iter(world, None) {
        if tx.send(buf.clone()).is_err() {
            log::warn!("Attempt to broadcast to disconnected player ")
        }
    }
}
