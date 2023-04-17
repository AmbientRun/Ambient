use ambient_ecs::{components, Debuggable, EntityId, Resource, World};

components!("wasm::message", {
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
    pub(super) module_id: Option<EntityId>,
    pub(super) source: Source,
    pub(super) name: String,
    pub(super) data: Vec<u8>,
}

pub fn send(
    world: &mut World,
    module_id: Option<EntityId>,
    source: Source,
    name: String,
    data: Vec<u8>,
) {
    world
        .resource_mut(pending_messages())
        .push(SerializedMessage {
            module_id,
            source,
            name,
            data,
        });
}

pub(super) fn run(
    world: &mut World,
    SerializedMessage {
        module_id,
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

    if let Some(module_id) = module_id {
        match world.get_cloned(module_id, module_state()) {
            Ok(state) => super::run(world, module_id, state, &source, &name, &data),
            Err(_) => {
                let module_name = world
                    .get_cloned(module_id, ambient_core::name())
                    .unwrap_or_default();

                world.resource(messenger()).as_ref()(
                    world, module_id, MessageType::Warn,
                    &format!("Received message for unloaded module {module_id} ({module_name}); message {name:?} from {source:?}")
                );
            }
        }
    } else {
        for (id, sms) in query(module_state()).collect_cloned(world, None) {
            if Some(id) == source_id {
                continue;
            }

            super::run(world, id, sms, &source, &name, &data)
        }
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
                module_id,
                source: Source::Runtime,
                name: T::id().to_string(),
                data: self.serialize_message()?,
            },
        );
        Ok(())
    }
}
