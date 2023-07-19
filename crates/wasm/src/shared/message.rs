use super::module_name;
use ambient_ecs::{components, Debuggable, EntityId, Resource, World};

components!("wasm/message", {
    @[Debuggable, Resource]
    pending_messages: Vec<SerializedMessage>,
});

#[derive(Clone, PartialEq, Debug)]
pub enum Source {
    Runtime,
    Server,
    Client(String),
    Local(EntityId),
}

#[derive(Clone, PartialEq, Debug)]
pub struct SerializedMessage {
    /// If unspecified, this will broadcast to all modules
    pub(super) target: Target,
    pub(super) source: Source,
    pub(super) name: String,
    pub(super) data: Vec<u8>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Target {
    /// Send to all modules on this side
    All { include_self: bool },
    /// Send to a specific module on this side
    Module(EntityId),
}

pub fn send(world: &mut World, target: Target, source: Source, name: String, data: Vec<u8>) {
    world
        .resource_mut(pending_messages())
        .push(SerializedMessage {
            target,
            source,
            name,
            data,
        });
}

pub(super) fn run(
    world: &mut World,
    SerializedMessage {
        target,
        source,
        name,
        data,
    }: SerializedMessage,
) {
    use super::{messenger, module_state, MessageType};
    use ambient_ecs::query;

    let source_id = if let Source::Local(id) = &source {
        Some(*id)
    } else {
        None
    };

    match target {
        Target::All { include_self } => {
            for (id, sms) in query(module_state()).collect_cloned(world, None) {
                if Some(id) == source_id && !include_self {
                    continue;
                }

                super::run(world, id, sms, &source, &name, &data)
            }
        }
        Target::Module(module_id) => match world.get_cloned(module_id, module_state()) {
            Ok(state) => super::run(world, module_id, state, &source, &name, &data),
            Err(_) => {
                let module_name = world
                    .get_cloned(module_id, module_name())
                    .unwrap_or_default();

                world.resource(messenger()).as_ref()(
                    world, module_id, MessageType::Warn,
                    &format!("Received message for unloaded module {module_id} ({module_name}); message {name:?} from {source:?}")
                );
            }
        },
    }
}

pub trait RuntimeMessageExt {
    fn run(self, world: &mut World, module_id: Option<EntityId>) -> anyhow::Result<()>;
}
impl<T: ambient_ecs::RuntimeMessage> RuntimeMessageExt for T {
    fn run(self, world: &mut World, module_id: Option<EntityId>) -> anyhow::Result<()> {
        run(
            world,
            SerializedMessage {
                target: module_id
                    .map(Target::Module)
                    .unwrap_or(Target::All { include_self: true }),
                source: Source::Runtime,
                name: T::id().to_string(),
                data: self.serialize_message()?,
            },
        );
        Ok(())
    }
}
