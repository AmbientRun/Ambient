pub mod logic;
mod registry;

use std::{fmt::Debug, sync::Arc};

use ambient_ecs::{
    components, index_system, query, ArchetypeFilter, Component, ComponentValue, Debuggable,
    Entity, EntityId, Index, IndexColumns, Networked, QueryState, Resource, Store, SystemGroup,
};
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use ambient_network::{
    client::GameClient,
    hooks::{use_remote_component, use_remote_world_system},
    server::{RpcArgs as ServerRpcArgs, SharedServerState},
    unwrap_log_network_err,
};
use ambient_rpc::RpcRegistry;
use ambient_ui_native::{FlowColumn, StylesExt, Text};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use logic::{create_intent, push_intent, redo_intent, undo_head, undo_head_exact};
pub use registry::*;

components!("intent", {
    /// The component index of the intent
    @[Debuggable, Networked, Store]
    intent: u32,

    /// Intents with the same id and intent type "next to" each other will be collapsed.
    ///
    /// It is an error for two events of the same id and incompatible types
    @[Debuggable, Networked, Store]
    intent_id: String,
    @[Debuggable, Networked, Store]
    intent_timestamp: DateTime<Utc>,
    @[Debuggable, Networked, Store]
    intent_user_id: String,
    @[Debuggable, Networked, Store]
    intent_reverted: (),
    /// Set for an applied intent, with a debug format of the revert intent
    @[Debuggable, Networked, Store]
    intent_applied: String,
    @[Debuggable, Networked, Store]
    intent_failed: String,
    @[Debuggable, Networked, Store]
    intent_success: (),
    @[Debuggable, Networked, Store]
    intent_no_state: (),

    @[Debuggable, Resource]
    intent_registry: Arc<IntentRegistry>,

    // Index
    @[Debuggable, Resource]
    intent_index: Index,
    @[Debuggable]
    intent_id_index: Index,
    @[Debuggable, Resource]
    intent_index_reverted: Index,
    @[Debuggable, Resource]
    intent_index_applied: Index,
});

pub async fn client_push_intent<T: ComponentValue>(
    game_client: GameClient,
    intent_arg: Component<T>,
    arg: T,
    collapse_id: Option<String>,
    on_applied: Option<Box<dyn Fn() + Sync + Send + 'static>>,
) {
    let ed = create_intent(intent_arg, arg, collapse_id);
    let intent_id = unwrap_log_network_err!(game_client.rpc(rpc_push_intent, ed).await);
    if let Some(on_applied) = on_applied {
        if let Some(intent_id) = intent_id {
            let mut state = game_client.game_state.lock();
            let mut qs = QueryState::new();
            state.add_temporary_system(move |world| {
                for (id, _) in query(intent_applied()).spawned().iter(world, Some(&mut qs)) {
                    if id == intent_id {
                        on_applied();
                        return true;
                    }
                }
                false
            });
        }
    }
}

pub async fn server_push_intent<T: ComponentValue>(
    state: SharedServerState,
    intent_arg: Component<T>,
    arg: T,
    user_id: String,
    collapse_id: Option<String>,
) {
    push_intent(state, user_id, create_intent(intent_arg, arg, collapse_id));
}

pub async fn rpc_push_intent(args: ServerRpcArgs, intent: Entity) -> Option<EntityId> {
    Some(push_intent(args.state, args.user_id, intent))
}

pub async fn rpc_undo_head(args: ServerRpcArgs, _: ()) -> Option<()> {
    undo_head(args.state, &args.user_id)?;
    Some(())
}

/// Reverts the head intent iff it is the specified intent
pub async fn rpc_undo_head_exact(args: ServerRpcArgs, id: String) -> Option<()> {
    undo_head_exact(args.state, &args.user_id, &id)?;

    Some(())
}

pub async fn rpc_redo(args: ServerRpcArgs, _: ()) -> Option<()> {
    let state = args.state;
    redo_intent(state, &args.user_id).await?;
    Some(())
}

pub fn register_server_rpcs(reg: &mut RpcRegistry<ServerRpcArgs>) {
    reg.register(rpc_push_intent);
    reg.register(rpc_undo_head);
    reg.register(rpc_undo_head_exact);
    reg.register(rpc_redo);
}

pub fn common_intent_systems() -> SystemGroup {
    SystemGroup::new(
        "intents/common",
        vec![
            Box::new(index_system(
                ArchetypeFilter::new().excl(intent_reverted()),
                IndexColumns::new().add_column(intent_id()),
                intent_id_index(),
            )),
            Box::new(index_system(
                ArchetypeFilter::new().excl(intent_reverted()),
                IndexColumns::new()
                    .add_column(intent_user_id())
                    .add_column(intent_timestamp()),
                intent_index(),
            )),
            Box::new(index_system(
                ArchetypeFilter::new().incl(intent_reverted()),
                IndexColumns::new()
                    .add_column(intent_user_id())
                    .add_column(intent_timestamp()),
                intent_index_reverted(),
            )),
            Box::new(index_system(
                ArchetypeFilter::new()
                    .excl(intent_reverted())
                    .incl(intent_applied()),
                IndexColumns::new()
                    .add_column(intent_user_id())
                    .add_column(intent_timestamp()),
                intent_index_applied(),
            )),
        ],
    )
}

#[derive(Debug, Clone)]
pub struct IntentHistoryVisualizer;
impl ElementComponent for IntentHistoryVisualizer {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let (intents, set_intents) = hooks.use_state(Vec::new());
        use_remote_world_system(
            hooks,
            query(()).incl(intent_user_id()),
            move |q, world, qs, _| {
                set_intents(
                    q.iter(world, qs)
                        .sorted_by_key(|(id, _)| world.get(*id, intent_timestamp()).ok())
                        .map(|(id, _)| id)
                        .collect(),
                );
            },
        );
        FlowColumn::el(
            intents
                .into_iter()
                .map(|intent| IntentVisualizer { id: intent }.el())
                .collect_vec(),
        )
        .floating_panel()
    }
}

#[derive(Debug, Clone)]
pub struct IntentVisualizer {
    id: EntityId,
}
impl ElementComponent for IntentVisualizer {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { id } = *self;

        let intent = use_remote_component(hooks, id, intent());
        let timestamp = use_remote_component(hooks, id, intent_timestamp());
        let collapse_id = use_remote_component(hooks, id, intent_id());
        let reverted = use_remote_component(hooks, id, intent_reverted());
        let applied = use_remote_component(hooks, id, intent_applied());
        let failed = use_remote_component(hooks, id, intent_failed());

        Text::el(format!(
            "{:?} {:?} applied={:?} reverted={} {:?} {}",
            intent,
            timestamp,
            applied,
            reverted.is_ok(),
            collapse_id.ok(),
            failed.unwrap_or(String::new())
        ))
    }
}

/// Helper functions for collapsing absolute state intents
pub fn use_old_state<T: Clone + Debug, U: Clone + Debug>(
    _old_arg: &T,
    old_state: &U,
    new_arg: &T,
    _new_state: &U,
) -> (T, U) {
    (new_arg.clone(), old_state.clone())
}
