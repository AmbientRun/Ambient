pub use ambient_ecs::generated::components::core::wasm::message::*;
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
    Module(EntityId),
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
    use super::{messenger, module_state, MessageType, RunContext};
    use ambient_ecs::{query, Entity};

    let mut entity = Entity::new().with(self::data(), data.to_vec());

    let mut source_id = None;
    match &source {
        Source::Runtime => entity.set(source_runtime(), ()),
        Source::Server => entity.set(source_server(), ()),
        Source::Client(user_id) => entity.set(source_client_user_id(), user_id.to_owned()),
        Source::Module(id) => {
            source_id = Some(*id);
            entity.set(source_local(), *id);
        }
    };

    let run_context = RunContext::new(
        world,
        format!("{}/{}", ambient_shared_types::events::MODULE_MESSAGE, name),
        entity,
    );

    if let Some(module_id) = module_id {
        match world.get_cloned(module_id, module_state()) {
            Ok(state) => super::run(world, module_id, state, &run_context),
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

            super::run(world, id, sms, &run_context)
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
