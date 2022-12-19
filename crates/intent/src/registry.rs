use std::{collections::HashMap, fmt::Debug, marker::Send};

use elements_ecs::{
    with_component_registry, ArchetypeFilter, Component, ComponentValue, EntityData, EntityId, IComponent, IndexExt, SystemGroup, World
};
use elements_network::server::{ServerState, SharedServerState};
use futures::Future;
use parking_lot::MutexGuard;
use tracing::info_span;

use crate::{
    common_intent_systems, intent, intent_applied, intent_failed, intent_id, intent_id_index, intent_index, intent_index_applied, intent_index_reverted, intent_reverted, intent_success, logic::get_head_applied_intent
};

pub struct IntentContext<'a> {
    pub world: &'a mut World,
    pub user_id: &'a str,
}

impl<'a> IntentContext<'a> {
    fn from_guard(guard: &'a mut MutexGuard<'_, ServerState>, user_id: &'a str) -> IntentContext<'a> {
        IntentContext { world: guard.get_player_world_mut(user_id).expect("Missing player world"), user_id }
    }
}

pub trait IntentFn<'a, Arg, R>: 'static + Send + Sync {
    type Future: 'a + Future<Output = anyhow::Result<R>> + Send;
    fn call(&self, ctx: &'a mut IntentContext<'_>, arg: Arg) -> Self::Future;
}

impl<'a, F, Arg, R, Fut> IntentFn<'a, Arg, R> for F
where
    F: Fn(&'a mut IntentContext<'_>, Arg) -> Fut + 'static + Send + Sync,
    Fut: 'a + Send + Future<Output = anyhow::Result<R>>,
{
    type Future = Fut;

    fn call(&self, ctx: &'a mut IntentContext<'_>, arg: Arg) -> Self::Future {
        (self)(ctx, arg)
    }
}

/// Function which returns a future bound by its argument lifetime
pub trait BorrowedGenerator<'a, Arg, Ret>: 'static + Send + Sync
where
    Ret: 'a + Send,
    Arg: 'a,
{
    type Output: Future<Output = Ret> + Send + 'a;
    fn call(&self, arg: Arg) -> Self::Output;
}

impl<'a, Arg, Ret, Fut, F> BorrowedGenerator<'a, Arg, Ret> for F
where
    F: Fn(Arg) -> Fut + Send + Sync + 'static,
    Arg: 'a,
    Ret: 'a + Send,
    Fut: Future<Output = Ret> + Send + 'static,
{
    type Output = Fut;
    fn call(&self, arg: Arg) -> Fut {
        (self)(arg)
    }
}

trait Handler<'a>: 'static + Send + Sync {
    fn name(&self) -> &str;
    fn apply(&'a self, ctx: IntentContext<'a>, id: EntityId);
    fn revert(&'a self, ctx: IntentContext<'a>, id: EntityId);
    fn merge(&self, ctx: &mut IntentContext<'_>, a: EntityId, b: EntityId);
}

pub struct IntentHandler<Arg: ComponentValue, RevertState: ComponentValue, Apply, Revert, Merge> {
    intent: Component<Arg>,
    intent_revert: Component<RevertState>,
    name: String,
    apply: Apply,
    revert: Revert,
    merge: Merge,
}

impl<'a, Arg, RevertState, Apply, Revert, Merge> Handler<'a> for IntentHandler<Arg, RevertState, Apply, Revert, Merge>
where
    Apply: for<'x> Fn(IntentContext<'_>, Arg) -> anyhow::Result<RevertState> + Send + Sync + 'static,
    Revert: for<'x> Fn(IntentContext<'_>, RevertState) -> anyhow::Result<()> + Send + Sync + 'static,
    Merge: for<'x> Fn(&Arg, &RevertState, &Arg, &RevertState) -> (Arg, RevertState) + Send + Sync + 'static,
    //
    Arg: ComponentValue,
    RevertState: ComponentValue + Debug,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn apply(&'a self, mut ctx: IntentContext<'a>, id: EntityId) {
        let arg = ctx.world.get_ref(id, self.intent).expect("Intent is missing intent arg").clone();
        let head = get_head_applied_intent(ctx.world, ctx.user_id);
        let intent_arg = *ctx.world.get_ref(id, intent()).unwrap();
        if let Some(head) = head {
            assert!(ctx.world.exists(head), "Head does not exist");
        }
        let result = (self.apply)(IntentContext { world: ctx.world, user_id: ctx.user_id }, arg);
        let world = &mut ctx.world;
        if world.has_component(id, intent_applied()) {
            panic!("Intent applied twice");
        }

        match result {
            Ok(state) => {
                world
                    .add_components(
                        id,
                        EntityData::new()
                            .set(intent_applied(), format!("{state:?}"))
                            .set(intent_success(), ())
                            .set(self.intent_revert, state),
                    )
                    .unwrap();
            }
            Err(err) => {
                tracing::error!("Failed to apply intent: {id}\n\n{err:?}");
                world
                    .add_components(
                        id,
                        EntityData::new().set(intent_applied(), format!("failed: {:#}", err)).set(intent_failed(), format!("{:#}", err)),
                    )
                    .unwrap();
            }
        }

        let iid = world.get_ref(id, intent_id()).unwrap();

        if let Some(head) = head {
            let head_id = world.get_ref(head, intent_id()).unwrap();
            // let mut s = Vec::new();
            // world.dump_entity(head, 2, &mut s);
            // let s = String::from_utf8_lossy(&s);
            // tracing::info!("Head:\n{s}");

            if world.has_component(head, intent_success()) && world.get(head, intent()).unwrap() == intent_arg && head_id == iid {
                self.merge(&mut ctx, head, id);

                let world = &mut ctx.world;
                world.despawn(head).unwrap();

                world.sync_index(intent_id_index(), head, ArchetypeFilter::new().excl(intent_reverted()));
                world.sync_index(intent_index(), head, ArchetypeFilter::new().excl(intent_reverted()));
                world.sync_index(intent_index_reverted(), head, ArchetypeFilter::new().incl(intent_reverted()));
                world.sync_index(intent_index_applied(), head, ArchetypeFilter::new().incl(intent_applied()).excl(intent_reverted()));
            }
        }

        let world = &mut ctx.world;
        // Update the indices
        world.sync_index(intent_id_index(), id, ArchetypeFilter::new().excl(intent_reverted()));
        world.sync_index(intent_index(), id, ArchetypeFilter::new().excl(intent_reverted()));
        world.sync_index(intent_index_reverted(), id, ArchetypeFilter::new().incl(intent_reverted()));
        world.sync_index(intent_index_applied(), id, ArchetypeFilter::new().incl(intent_applied()).excl(intent_reverted()));
    }

    fn revert(&'a self, mut ctx: IntentContext<'a>, id: EntityId) {
        let world = &mut ctx.world;

        let revert_state = world.get_ref(id, self.intent_revert).expect("Missing revert state for intent").clone();

        let result = (self.revert)(IntentContext { world: ctx.world, user_id: ctx.user_id }, revert_state);

        let name = &self.name;
        let world = &mut ctx.world;
        match result {
            Ok(()) => {
                world.add_components(id, EntityData::new().set(intent_reverted(), ())).unwrap();
            }
            Err(err) => {
                tracing::error!("Failed to revert intent: {name} {err:?}");
                world.add_components(id, EntityData::new().set(intent_reverted(), ()).set(intent_failed(), format!("{:#}", err))).unwrap();
            }
        }

        // Update the indices
        world.sync_index(intent_id_index(), id, ArchetypeFilter::new().excl(intent_reverted()));
        world.sync_index(intent_index(), id, ArchetypeFilter::new().excl(intent_reverted()));
        world.sync_index(intent_index_reverted(), id, ArchetypeFilter::new().incl(intent_reverted()));
        world.sync_index(intent_index_applied(), id, ArchetypeFilter::new().incl(intent_applied()).excl(intent_reverted()));
    }

    fn merge(&self, ctx: &mut IntentContext<'_>, a: EntityId, b: EntityId) {
        let _span = info_span!("merge_intent", self.name).entered();
        let world = &mut ctx.world;
        let old_arg = world.get_ref(a, self.intent).expect("Missing old intent");
        let old_state = world.get_ref(a, self.intent_revert).expect("Old intent not applied");

        let new_arg = world.get_ref(b, self.intent).expect("Missing new intent");
        let new_state = world.get_ref(b, self.intent_revert).expect("New intent not applied");

        let (arg, state) = { (self.merge)(old_arg, old_state, new_arg, new_state) };

        *world.get_mut(b, intent_applied()).unwrap() = format!("{state:?}");
        *world.get_mut(b, self.intent).unwrap() = arg;
        *world.get_mut(b, self.intent_revert).unwrap() = state;
    }
}

pub struct IntentRegistry {
    handlers: HashMap<usize, Box<dyn for<'x> Handler<'x>>>,
}

impl Debug for IntentRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_map();
        for (k, v) in &self.handlers {
            s.entry(k, &v.name());
        }

        s.finish()
    }
}

impl IntentRegistry {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// Register a new intent.
    ///
    /// The apply function takes an intent and applies it to the world. The returned value should
    /// reflect the non-relative/non-dependent state to revert to the state of the world prior to
    /// the application of intents.
    ///
    /// The revert function uses the state returned by a successful apply to undo the effect by
    /// apply. Note that the revert function does not have access to the initial intent, this is
    /// because this value may not be up-to-date if intents are merged, as it usually corresponds
    /// to the top-level intent, and not the merged stack as a whole.
    ///
    /// Intents may be merged, in which only the most recent intent will be applied. The undo state
    /// will be of the oldest intent. As such, reverting a merged intent will revert to the state
    /// before the first/oldest of the merged intents.
    /// # Merge
    /// Function determining how to reconcile two (applied) intents that are to be merged.
    ///
    /// Takes in the head argument (which will be despawned), the recently applied intent.
    /// The intents will then be replaced by a single intent with the returned
    /// value.
    ///
    /// **Note**: The lock on the world will be held for the whole duration of the async intent.
    ///
    /// This is to prevent deadlocks and swept under the rug when intents are partially applied.
    /// If this is not desired, consider [`tokio::spawn`] or [`elements_core::AsyncRun`].
    pub fn register<Arg, RevertState, Apply, Revert, Merge>(
        &mut self,
        intent: Component<Arg>,
        intent_revert: Component<RevertState>,
        apply: Apply,
        revert: Revert,
        merge: Merge,
    ) where
        Apply: for<'x> Fn(IntentContext<'_>, Arg) -> anyhow::Result<RevertState> + Send + Sync + 'static,
        Revert: for<'x> Fn(IntentContext<'_>, RevertState) -> anyhow::Result<()> + Send + Sync + 'static,
        Merge: for<'x> Fn(&Arg, &RevertState, &Arg, &RevertState) -> (Arg, RevertState) + Send + Sync + 'static,
        //
        Arg: ComponentValue,
        RevertState: ComponentValue + Debug,
    {
        let idx_to_id = with_component_registry(|r| r.idx_to_id().clone());
        let name = idx_to_id.get(&intent.get_index()).cloned().unwrap_or_else(|| intent.get_index().to_string());
        let handler = IntentHandler { name, intent, intent_revert, apply, revert, merge };

        self.handlers.insert(intent.get_index(), Box::new(handler));
    }

    pub fn get_intent_name(&self, intent: usize) -> Option<String> {
        Some(self.handlers.get(&intent)?.name().to_string())
    }

    pub async fn apply_intent(&self, state: SharedServerState, intent_arg: usize, user_id: &str, id: EntityId) {
        let mut guard = state.lock();
        let ctx = IntentContext::from_guard(&mut guard, user_id);

        // Update the indices
        // self.index_systems.run(ctx.world, &FrameEvent);

        let head = get_head_applied_intent(ctx.world, user_id);
        if let Some(head) = head {
            assert!(ctx.world.exists(head), "Head intent does not exist");
        }

        // Check if it is possible to collapse the intents
        let handler = self.handlers.get(&intent_arg).expect("No handler for intent");

        handler.apply(ctx, id);

        // let world = ctx.world_mut();

        // self.index_systems.run(world, &FrameEvent);
    }

    /// Reverts an intent
    ///
    /// Intent must be previously applied
    pub async fn revert_intent(&self, state: SharedServerState, intent_arg: usize, user_id: &str, id: EntityId) {
        let mut guard = state.lock();
        let ctx = IntentContext::from_guard(&mut guard, user_id);

        let handler = self.handlers.get(&intent_arg).expect("No handler for intent");

        handler.revert(ctx, id)

        //         let world = ctx.world_mut();
        //         // Update the indices
        //         self.index_systems.run(world, &FrameEvent)
    }
}

pub fn registry_systems() -> SystemGroup {
    SystemGroup::new("intents/registry", vec![Box::new(common_intent_systems())])
}

#[cfg(test)]
mod test {

    use elements_ecs::{components, SimpleComponentRegistry};

    use super::*;

    #[test]
    fn registry() {
        SimpleComponentRegistry::install();
        components!("intent", {
           intent_add: f32,
           intent_add_revert: f32,
        });

        init_components();

        crate::init_components();
        let mut world = World::new("registry_test");

        let mut reg = IntentRegistry::new();
        reg.register(
            intent_add(),
            intent_add_revert(),
            |ctx, arg| todo!(),
            |ctx, revert| todo!(),
            |_, old_state, new_arg, _| (*old_state, *new_arg),
        );
    }
}
