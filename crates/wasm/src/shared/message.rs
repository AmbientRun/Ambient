use super::module_name;
pub use ambient_ecs::WorldEventSource;
use ambient_ecs::{
    components, generated::wasm::components::is_module, world_events, Debuggable, EntityId,
    Resource, World, WorldContext,
};
use ambient_package_semantic_native::{client_modules, is_package, server_modules};

components!("wasm::message", {
    @[Debuggable, Resource]
    pending_messages: Vec<SerializedMessage>,
});

#[derive(Clone, PartialEq, Debug)]
pub struct SerializedMessage {
    pub(super) target: Target,
    pub(super) source: WorldEventSource,
    pub(super) name: String,
    pub(super) data: Vec<u8>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Target {
    /// Send to all packages on this side
    All { include_self: bool },
    /// Send to a specific package or module on this side
    PackageOrModule(EntityId),
}

pub fn send(
    world: &mut World,
    target: Target,
    source: WorldEventSource,
    name: String,
    data: Vec<u8>,
) {
    if matches!(target, Target::All { .. }) {
        world
            .resource_mut(world_events())
            .add_event((source, name.clone(), data.clone()));
    } else {
        world
            .resource_mut(pending_messages())
            .push(SerializedMessage {
                target,
                source,
                name,
                data,
            });
    }
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

    let source_id = if let WorldEventSource::Local(id) = &source {
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
        Target::PackageOrModule(id) => {
            let is_module = world.has_component(id, is_module());
            let is_package = world.has_component(id, is_package());

            let warn = |msg| {
                world.resource(messenger()).as_ref()(world, id, MessageType::Warn, msg);
            };

            match (is_package, is_module) {
                (true, _) => {
                    let modules = match world.context() {
                        WorldContext::Server => world
                            .get_cloned(id, server_modules())
                            .ok()
                            .unwrap_or_default(),
                        WorldContext::Client => world
                            .get_cloned(id, client_modules())
                            .ok()
                            .unwrap_or_default(),
                        _ => {
                            let msg = format!("Received message {name:?} from {source:?} for package {id}, but the current world does not support WASM");
                            warn(&msg);
                            return;
                        }
                    };

                    for module_id in modules {
                        if let Ok(module_state) = world.get_cloned(module_id, module_state()) {
                            super::run(world, module_id, module_state, &source, &name, &data);
                        }
                    }
                }
                (_, true) => match world.get_cloned(id, module_state()) {
                    Ok(state) => super::run(world, id, state, &source, &name, &data),
                    Err(_) => {
                        let name = world.get_cloned(id, module_name()).unwrap_or_default();
                        let msg = format!("Received message {name:?} from {source:?} for unloaded module {id} ({name})");
                        warn(&msg);
                    }
                },
                (false, false) => {
                    let msg = format!("Received message for entity {id}, but entity is neither a package or a module; message {name:?} from {source:?}");
                    warn(&msg)
                }
            }
        }
    }
}

pub trait MessageExt {
    fn run(self, world: &mut World, module_id: Option<EntityId>) -> anyhow::Result<()>;
}
impl<T: ambient_ecs::Message> MessageExt for T {
    fn run(self, world: &mut World, module_id: Option<EntityId>) -> anyhow::Result<()> {
        run(
            world,
            SerializedMessage {
                target: module_id
                    .map(Target::PackageOrModule)
                    .unwrap_or(Target::All { include_self: true }),
                source: WorldEventSource::Runtime,
                name: T::id().to_string(),
                data: self.serialize_message()?,
            },
        );
        Ok(())
    }
}
