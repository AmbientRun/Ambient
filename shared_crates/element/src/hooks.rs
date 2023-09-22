#[cfg(feature = "native")]
use std::future::Future;
use std::{
    any::{type_name, TypeId},
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
    time::Duration,
};

use ambient_cb::{cb, Cb};
#[cfg(feature = "native")]
use ambient_core::runtime;
#[cfg(feature = "guest")]
use ambient_guest_bridge::ecs::EntityId;
#[cfg(feature = "native")]
use ambient_guest_bridge::ecs::{
    read_messages, world_events, ComponentQuery, FrameEvent, QueryState, TypedReadQuery,
};
use ambient_guest_bridge::{
    ecs::{ComponentValue, World},
    MessageContext, ModuleMessage, RuntimeMessage,
};
#[cfg(feature = "native")]
use ambient_sys::task;
use as_any::Downcast;
#[cfg(feature = "native")]
use atomic_refcell::AtomicRefCell;
use parking_lot::Mutex;
#[cfg(feature = "native")]
use tracing::info_span;

use crate::{AnyCloneable, ElementTree, HookContext, InstanceId};

/// Helper type for a callback that sets some value.
pub type Setter<T> = Cb<dyn Fn(T) + Sync + Send>;

type SpawnFn = Box<dyn FnOnce(&mut World) -> DespawnFn + Sync + Send>;
/// The return type of a function passed to [use_spawn]. This function is called
/// when the [Element](crate::Element) is unmounted/despawned; use it to clean up any resources.
pub type DespawnFn = Box<dyn FnOnce(&mut World) + Sync + Send>;

/// Hooks are a way to hook into the state and lifecycle of an [Element](crate::Element).
///
/// They are inspired by [React hooks](https://react.dev/learn#using-hooks).
///
/// Using hooks, you can store state, interact with the world, and generally do
/// operations that go beyond the pure rendering of an [Element](crate::Element).
pub struct Hooks<'a> {
    #[doc(hidden)]
    pub world: &'a mut World,
    pub(crate) instance_id: InstanceId,
    pub(crate) tree: &'a mut ElementTree,
    pub(crate) state_index: usize,
    pub(crate) on_spawn: Option<Vec<SpawnFn>>,
    pub(crate) environment: Arc<Mutex<HooksEnvironment>>,
}
impl<'a> Hooks<'a> {
    /// Instance ID is a unique ID for this instance of the component. It is used to identify the component
    /// for updates. It is not the same as the entity ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

/// Preserve state between rerenders.
///
/// The state can be mutated by the setter, which will re-render the origin [Element](crate::Element).
///
/// **Note**: The new value set by the returned setter won't be visible until the next
/// re-render.
///
/// ```rust,ignore
/// let (value, setter) = use_state(hooks,5);
///
/// setter(7);
///
/// println!("{value}"); // Prints 5
/// ```
pub fn use_state<T: Clone + Debug + Send + 'static>(hooks: &mut Hooks, init: T) -> (T, Setter<T>) {
    use_state_with(hooks, |_| init)
}

/// The same as [use_state], but with a function that returns the initial value.
///
/// This can be used to avoid cloning the initial value each time the [Element](crate::Element) is re-rendered.
pub fn use_state_with<T: Clone + Debug + Send + 'static>(
    hooks: &mut Hooks,
    init: impl FnOnce(&mut World) -> T,
) -> (T, Setter<T>) {
    let index = hooks.state_index;
    hooks.state_index += 1;
    let value = {
        let instance = hooks.tree.instances.get_mut(&hooks.instance_id).unwrap();
        if let Some(value) = instance.hooks_state.get(index) {
            value
        } else {
            instance.hooks_state.push(Box::new(init(hooks.world)));
            instance.hooks_state.last().unwrap()
        }
        .downcast_ref::<T>()
        .unwrap()
        .clone()
    };
    let environment = hooks.environment.clone();
    let element = hooks.instance_id.clone();
    (
        value,
        cb(move |new_value| {
            environment.lock().set_states.push(StateUpdate {
                instance_id: element.clone(),
                index,
                value: Box::new(new_value),
            })
        }),
    )
}

/// Provides a function that, when called, will cause this [Element](crate::Element) to be re-rendered.
// TODO: consider a more efficient implementation?
pub fn use_rerender_signal(hooks: &mut Hooks) -> Cb<dyn Fn() + Sync + Send> {
    let (_, signal) = use_state(hooks, ());
    cb(move || signal(()))
}

/// Provide a value which is accessible to all children further down the
/// tree.
///
/// **Note**: This hook does not rely on order, and is therefore safe to use inside
/// conditionals.
pub fn provide_context<T: Clone + Debug + Send + Sync + 'static>(
    hooks: &mut Hooks,
    default_value: impl FnOnce() -> T,
) -> Setter<T> {
    let instance = hooks.tree.instances.get_mut(&hooks.instance_id).unwrap();
    let type_id = TypeId::of::<T>();
    instance
        .hooks_context_state
        .entry(type_id)
        .or_insert_with(|| HookContext {
            value: Box::new(default_value()),
            listeners: HashSet::new(),
        });
    let environment = hooks.environment.clone();
    let element = hooks.instance_id.clone();
    cb(move |new_value| {
        environment.lock().set_contexts.push(ContextUpdate {
            instance_id: element.clone(),
            type_id: TypeId::of::<T>(),
            name: type_name::<T>(),
            value: Box::new(new_value),
        });
    })
}
#[allow(clippy::type_complexity)]
/// Consume a context of type `T` provided further up the tree.
///
/// To provide context, use [provide_context].
pub fn consume_context<T: Clone + Debug + Sync + Send + 'static>(
    hooks: &mut Hooks,
) -> Option<(T, Setter<T>)> {
    let type_id = TypeId::of::<T>();
    if let Some(provider) = hooks.tree.get_context_provider(&hooks.instance_id, type_id) {
        let value = {
            let instance = hooks.tree.instances.get_mut(&provider).unwrap();
            let ctx = instance.hooks_context_state.get_mut(&type_id).unwrap();
            ctx.listeners.insert(hooks.instance_id.clone());
            ctx.value.downcast_ref::<T>().unwrap().clone()
        };
        {
            let instance = hooks.tree.instances.get_mut(&hooks.instance_id).unwrap();
            instance
                .hooks_context_listening
                .insert((provider.clone(), type_id));
        }
        let environment = hooks.environment.clone();
        Some((
            value,
            cb(move |new_value| {
                environment.lock().set_contexts.push(ContextUpdate {
                    instance_id: provider.clone(),
                    type_id,
                    name: type_name::<T>(),
                    value: Box::new(new_value),
                });
            }),
        ))
    } else {
        None
    }
}

/// Execute a function when the [Element](crate::Element) is mounted/rendered for the first time.
///
/// The function should return another function; that function will be called when the [Element](crate::Element) is unmounted.
#[tracing::instrument(level = "debug", skip_all)]
pub fn use_spawn<
    F: FnOnce(&mut World) -> R + Sync + Send + 'static,
    R: FnOnce(&mut World) + Sync + Send + 'static,
>(
    hooks: &mut Hooks,
    func: F,
) {
    if let Some(ref mut on_spawn) = hooks.on_spawn {
        let spawn_fn: SpawnFn = Box::new(move |w| Box::new(func(w)));
        on_spawn.push(spawn_fn);
    }
}

/// Register a function to be called when a [RuntimeMessage] is received.
///
/// The subscription will be automatically cancelled when this [Element](crate::Element) is unmounted.
pub fn use_runtime_message<T: RuntimeMessage>(
    hooks: &mut Hooks,
    func: impl Fn(&mut World, &T) + Sync + Send + 'static,
) {
    #[cfg(feature = "native")]
    {
        let reader = use_ref_with(hooks, |world| world.resource(world_events()).reader());
        use_frame(hooks, move |world| {
            let mut reader = reader.lock();
            for event in read_messages(&mut reader, world.resource(world_events())) {
                func(world, &event);
            }
        })
    }
    #[cfg(feature = "guest")]
    {
        let handler = use_ref_with(hooks, |_| None);
        *handler.lock() = Some(cb(func));
        use_effect(hooks, (), move |_, _| {
            let listener = T::subscribe(move |event| {
                (handler.lock().as_ref().unwrap())(&mut World, &event);
            });
            move |_| listener.stop()
        });
    }
}

/// Register a function to be called when a [ModuleMessage] is received.
///
/// The subscription will be automatically cancelled when this [Element](crate::Element) is unmounted.
pub fn use_module_message<T: ModuleMessage>(
    hooks: &mut Hooks,
    func: impl Fn(&mut World, MessageContext, &T) + Sync + Send + 'static,
) {
    #[cfg(feature = "native")]
    {
        let reader = use_ref_with(hooks, |world| world.resource(world_events()).reader());
        use_frame(hooks, move |world| {
            let mut reader = reader.lock();
            for event in read_messages(&mut reader, world.resource(world_events())) {
                func(world, (), &event);
            }
        })
    }
    #[cfg(feature = "guest")]
    {
        let handler = use_ref_with(hooks, |_| None);
        *handler.lock() = Some(cb(func));
        use_effect(hooks, (), move |_, _| {
            let listener = T::subscribe(move |ctx, event| {
                (handler.lock().as_ref().unwrap())(&mut World, ctx, &event);
            });
            move |_| listener.stop()
        });
    }
}

/// Send the `Enter` message when this `Element` is mounted, and the `Exit` message when it is unmounted.
///
/// If the `target_id` is `Some`, the message will be directed to that target; otherwise, it will be broadcast.
#[cfg(feature = "guest")]
pub fn use_module_message_effect<Enter: ModuleMessage + Default, Exit: ModuleMessage + Default>(
    hooks: &mut Hooks,
    target_id: Option<EntityId>,
) {
    use ambient_guest_bridge::Target;

    use_effect(hooks, target_id, move |_, id| {
        let target = match *id {
            Some(id) => Target::Local(id),
            None => Target::LocalBroadcast { include_self: true },
        };
        Enter::default().send(target.clone());
        |_| Exit::default().send(target)
    });
}

/// Spawns the provided future as a task.
///
/// The task is aborted when this [Element](crate::Element) is removed.
#[cfg(feature = "native")]
#[tracing::instrument(level = "debug", skip_all)]
pub fn use_task<Fut: Future<Output = ()> + Send + 'static>(
    hooks: &mut Hooks,
    task: impl FnOnce(&mut World) -> Fut + Send + Sync + 'static,
) {
    if let Some(ref mut on_spawn) = hooks.on_spawn {
        let spawn = Box::new(move |w: &mut World| {
            let task = task(w);
            let task = w.resource(runtime()).spawn(task);
            Box::new(move |_: &mut World| task.abort()) as Box<dyn FnOnce(&mut World) + Sync + Send>
        });

        on_spawn.push(spawn)
    }
}

/// Spawns the provided future as a task.
///
/// The task is aborted when this [Element](crate::Element) is removed.
#[cfg(target_os = "unknown")]
#[cfg(feature = "native")]
#[tracing::instrument(level = "debug", skip_all)]
pub fn use_local_task<Fut: Future<Output = ()> + 'static>(
    hooks: &mut Hooks,
    task: impl FnOnce(&mut World) -> Fut + Send + Sync + 'static,
) {
    if let Some(ref mut on_spawn) = hooks.on_spawn {
        let spawn = Box::new(move |w: &mut World| {
            let task = task(w);
            let task = w.resource(runtime()).spawn_local(task);
            Box::new(move |_: &mut World| task.abort()) as Box<dyn FnOnce(&mut World) + Sync + Send>
        });

        on_spawn.push(spawn)
    }
}
/// Use a value provided by a future.
///
/// Returns `None` until the future completes.
///
/// Automatically triggers a re-render on this [Element](crate::Element) when the future completes.
#[cfg(feature = "native")]
pub fn use_async<T, U>(
    hooks: &mut Hooks,
    future: impl FnOnce(&mut World) -> T + Send + Sync + 'static,
) -> Option<U>
where
    T: Future<Output = U> + Send + 'static,
    U: Debug + ComponentValue,
{
    let (state, set_state) = use_state(hooks, None);
    use_spawn(hooks, |w| {
        let future = future(w);
        let runtime = w.resource(runtime());

        info_span!("use_async_spawn");
        let task = runtime.spawn(async move {
            let res = future.await;
            set_state(Some(res))
        });

        move |_| task.abort()
    });

    state
}

/// Use memoized state dependent on a future
#[cfg(feature = "native")]
pub fn use_memo_async<T, F, D, U>(hooks: &mut Hooks, deps: D, init: F) -> Option<T>
where
    F: FnOnce(&mut World, D) -> U,
    U: 'static + Future<Output = T> + Send,
    T: ComponentValue,
    D: PartialEq + ComponentValue,
{
    struct State<T, D> {
        task: AtomicRefCell<Option<task::JoinHandle<()>>>,
        value: Mutex<Option<T>>,
        prev_deps: AtomicRefCell<Option<D>>,
    }

    impl<T, D> Debug for State<T, D> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("State")
                .field("task", &"...")
                .field("value", &type_name::<T>())
                .field("prev_deps", &type_name::<D>())
                .finish()
        }
    }

    let (state, set_state) = use_state_with(hooks, |_| {
        Arc::new(State {
            task: AtomicRefCell::new(None),
            value: Mutex::new(None),
            prev_deps: AtomicRefCell::new(None),
        })
    });

    let mut prev_deps = state.prev_deps.borrow_mut();
    if prev_deps.as_ref() != Some(&deps) {
        let runtime = hooks.world.resource(runtime()).clone();
        // Update state
        // Cancel the previous calculation
        let mut task = state.task.borrow_mut();
        if let Some(task) = task.take() {
            task.abort();
        }

        let fut = init(hooks.world, deps.clone());
        *prev_deps = Some(deps);
        let state = state.clone();
        // The future may complete immediately
        *task = Some(runtime.spawn(async move {
            let value = fut.await;

            // Update the value
            *state.value.lock() = Some(value);
            set_state(state)
        }));
    } else {
        // A value may be available, but nothing is certain
    }

    let x = state.value.lock().as_ref().cloned();
    x
}

#[profiling::function]
/// Executes a function each frame.
pub fn use_frame<F: Fn(&mut World) + Sync + Send + 'static>(hooks: &mut Hooks, on_frame: F) {
    let mut env = hooks.environment.lock();
    let listeners = env
        .frame_listeners
        .entry(hooks.instance_id.clone())
        .or_insert_with(Vec::new);
    listeners.push(FrameListener(Arc::new(on_frame)));
}

/// Provides internally mutable state that is preserved between re-renders.
///
/// This should be used over [use_state] when reference semantics are required for the state
/// as opposed to value semantics.
///
/// **Note**: Locking the mutex and modifying the value won't cause a re-render. To re-render the element,
/// use [use_rerender_signal].
pub fn use_ref_with<T: Send + Debug + 'static>(
    hooks: &mut Hooks,
    init: impl FnOnce(&mut World) -> T,
) -> Arc<Mutex<T>> {
    use_state_with(hooks, |world| Arc::new(Mutex::new(init(world)))).0
}

#[profiling::function]

/// A computation that runs once on spawn, and when `dependencies` change. The computation is not re-run
/// if the `dependencies` are the same - that is, the computation is [memoized](https://en.wikipedia.org/wiki/Memoization).
///
/// **Note**: using external captures for the `create` function will not cause the memoized
/// value to be recalculated when the captures change.
///
/// Prefer to route as much as possible through the `dependencies`; these dependencies are available as arguments to `compute`.
pub fn use_memo_with<
    T: Clone + ComponentValue + Debug + Sync + Send + 'static,
    D: PartialEq + Clone + Sync + Send + Debug + 'static,
>(
    hooks: &mut Hooks,
    dependencies: D,
    compute: impl FnOnce(&mut World, &D) -> T,
) -> T {
    let value = use_ref_with(hooks, |_| None);
    let prev_deps = use_ref_with(hooks, |_| None);

    let mut prev_deps = prev_deps.lock();
    let mut value = value.lock();

    if prev_deps.as_ref() != Some(&dependencies) {
        let value = value.insert(compute(hooks.world, &dependencies)).clone();
        *prev_deps = Some(dependencies);
        value
    } else {
        value.clone().expect("No memo value")
    }
}

#[profiling::function]
/// Run a function for its side effects each time a dependency changes.
///
/// The function should return another function; that function will be called when the [Element](crate::Element) is unmounted.
pub fn use_effect<
    D: PartialEq + Debug + Sync + Send + 'static,
    R: FnOnce(&mut World) + Sync + Send + 'static,
>(
    hooks: &mut Hooks,
    dependencies: D,
    run: impl FnOnce(&mut World, &D) -> R + Sync + Send,
) {
    struct Cleanup(Box<dyn FnOnce(&mut World) + Sync + Send>);
    impl Debug for Cleanup {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Cleanup").finish()
        }
    }
    let cleanup_prev: Arc<Mutex<Option<Cleanup>>> = use_ref_with(hooks, |_| None);
    let prev_deps = use_ref_with::<Option<D>>(hooks, |_| None);
    {
        let cleanup_prev = cleanup_prev.clone();
        use_spawn(hooks, move |_| {
            move |world| {
                let mut cleanup_prev = cleanup_prev.lock();
                if let Some(cleanup_prev) = cleanup_prev.take() {
                    cleanup_prev.0(world);
                }
            }
        });
    }

    let dependencies = dependencies;
    let mut prev_deps = prev_deps.lock();
    if prev_deps.as_ref() != Some(&dependencies) {
        let mut cleanup_prev = cleanup_prev.lock();
        if let Some(cleanup_prev) = (*cleanup_prev).take() {
            cleanup_prev.0(hooks.world);
        }
        profiling::scope!("use_effect_run");
        *cleanup_prev = Some(Cleanup(Box::new(run(hooks.world, &dependencies))));
        *prev_deps = Some(dependencies);
    }
}

#[cfg(feature = "native")]
/// Run a native system each frame
pub fn use_system<
    'b,
    R: ComponentQuery<'b> + Clone + 'static,
    F: Fn(&TypedReadQuery<R>, &mut World, Option<&mut QueryState>, &FrameEvent)
        + Send
        + Sync
        + 'static,
>(
    hooks: &mut Hooks,
    query: TypedReadQuery<R>,
    run: F,
) {
    let query_state = use_ref_with(hooks, |_| QueryState::new());
    use_frame(hooks, move |world| {
        let mut qs = query_state.lock();
        run(&query, world, Some(&mut qs), &FrameEvent);
    });
}

#[cfg(feature = "guest")]
/// Query the ECS for entities with the given components each frame and return the results.
///
/// The components to query are anything that can be passed to [query](ambient_guest_bridge::api::prelude::query).
///
/// This can be used to track the state of entities in the ECS within an element.
pub fn use_query<
    Components: ambient_guest_bridge::api::ecs::ComponentsTuple + Debug + Copy + Clone + Sync + Send + 'static,
>(
    hooks: &mut Hooks,
    components: Components,
) -> Vec<(ambient_guest_bridge::ecs::EntityId, Components::Data)> {
    use ambient_guest_bridge::api::prelude::{change_query, despawn_query, query, spawn_query};

    let refresh = use_rerender_signal(hooks);
    let (query, _) = use_state_with(hooks, |_| query(components).build());
    use_spawn(hooks, move |_| {
        let s = spawn_query(components).bind({
            let refresh = refresh.clone();
            move |_| refresh()
        });
        let d = despawn_query(components).bind({
            let refresh = refresh.clone();
            move |_| refresh()
        });
        let c = change_query(components).track_change(components).bind({
            let refresh = refresh.clone();
            move |_| refresh()
        });
        move |_| {
            s.stop();
            d.stop();
            c.stop();
        }
    });
    query.evaluate()
}

#[cfg(feature = "guest")]
/// Use a component from an entity in the ECS.
///
/// If the entity or component does not exist, this will return `None`.
pub fn use_entity_component<
    T: ambient_guest_bridge::api::ecs::SupportedValue
        + Clone
        + Debug
        + Sync
        + Send
        + PartialEq
        + 'static,
>(
    hooks: &mut Hooks,
    id: EntityId,
    component: ambient_guest_bridge::api::ecs::Component<T>,
) -> Option<T> {
    use ambient_guest_bridge::api::prelude::{change_query, entity};

    let refresh = use_rerender_signal(hooks);
    use_spawn(hooks, move |_| {
        let c = change_query(component).track_change(component).bind({
            let refresh = refresh.clone();
            move |_| refresh()
        });
        move |_| {
            c.stop();
        }
    });

    entity::get_component(id, component)
}

#[cfg(feature = "guest")]
/// Use a resource from the ECS, and update its state if required.
///
/// If the resource does not exist, this will return `None`.
/// The setter will add the resource if it does not exist.
pub fn use_resource<
    T: ambient_guest_bridge::api::ecs::SupportedValue
        + Clone
        + Debug
        + Sync
        + Send
        + PartialEq
        + 'static,
>(
    hooks: &mut Hooks,
    component: ambient_guest_bridge::api::ecs::Component<T>,
) -> (Option<T>, Setter<T>) {
    use ambient_guest_bridge::api::entity;

    (
        use_entity_component(hooks, entity::resources(), component),
        cb(move |value| entity::add_component(entity::resources(), component, value)),
    )
}

#[cfg(feature = "guest")]
/// Use a concept from an entity in the ECS, and update its state if required.
///
/// If the entity does not exist, or does not match the concept, this will return `None`.
pub fn use_entity_concept<C: ambient_guest_bridge::api::ecs::ConceptComponents>(
    hooks: &mut Hooks,
    id: EntityId,
) -> Option<C> {
    use ambient_guest_bridge::api::prelude::change_query;

    let refresh = use_rerender_signal(hooks);
    use_spawn(hooks, move |_| {
        let c = change_query(())
            .track_change(C::required())
            .track_change(C::optional())
            .requires(C::required())
            .bind({
                let refresh = refresh.clone();
                move |_| refresh()
            });
        move |_| {
            c.stop();
        }
    });

    C::get_spawned(id)
}

/// Run `cb` every `seconds` seconds.
///
/// If your `cb` depends on some state, consider using [use_interval_deps] instead.
/// This function will capture the state at the time it is called, and will not update if the state changes.
pub fn use_interval<F: Fn() + Sync + Send + 'static>(hooks: &mut Hooks, seconds: f32, cb: F) {
    #[cfg(feature = "native")]
    use_spawn(hooks, move |world| {
        let thread = world.resource(runtime()).spawn(async move {
            let mut interval = ambient_sys::time::interval(Duration::from_secs_f32(seconds));
            interval.tick().await;
            loop {
                interval.tick().await;
                cb();
            }
        });
        move |_| {
            thread.abort();
        }
    });
    #[cfg(feature = "guest")]
    use_spawn(hooks, move |world| {
        use std::sync::atomic::AtomicBool;
        let exit = Arc::new(AtomicBool::new(false));
        {
            let exit = exit.clone();
            ambient_guest_bridge::run_async(world, async move {
                // TODO: This isn't a "true" interval, since it depends on how long cb takes, but the API doesn't support interavls yet
                while !exit.load(std::sync::atomic::Ordering::SeqCst) {
                    ambient_guest_bridge::sleep(seconds).await;
                    cb();
                }
            });
        }
        move |_| {
            exit.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

/// Run `cb` every `duration`, passing in the current value of `dependencies`.
///
/// This should be used when the callback depends on some value that can change over time.
pub fn use_interval_deps<D>(
    hooks: &mut Hooks,
    duration: Duration,
    run_immediately: bool,
    dependencies: D,
    mut func: impl 'static + Send + Sync + FnMut(&D),
) where
    D: 'static + Send + Sync + Clone + Debug + PartialEq,
{
    #[cfg(feature = "native")]
    use_effect(hooks, dependencies.clone(), move |world, _| {
        if run_immediately {
            func(&dependencies);
        }

        let task = world.resource(runtime()).spawn(async move {
            let mut interval = ambient_sys::time::interval(duration);
            interval.tick().await;
            loop {
                interval.tick().await;
                func(&dependencies);
            }
        });

        move |_| {
            task.abort();
        }
    });
    #[cfg(feature = "guest")]
    use_effect(hooks, dependencies.clone(), move |world, _| {
        use std::sync::atomic::AtomicBool;
        if run_immediately {
            func(&dependencies);
        }

        let exit = Arc::new(AtomicBool::new(false));
        {
            let exit = exit.clone();
            ambient_guest_bridge::run_async(world, async move {
                // TODO: This isn't a "true" interval, since it depends on how long cb takes, but the API doesn't support interavls yet
                while !exit.load(std::sync::atomic::Ordering::SeqCst) {
                    ambient_guest_bridge::sleep(duration.as_secs_f32()).await;
                    func(&dependencies);
                }
            });
        }
        move |_| {
            exit.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[derive(Clone)]
pub(crate) struct FrameListener(pub Arc<dyn Fn(&mut World) + Sync + Send>);
impl Debug for FrameListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FrameListener").finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ContextUpdate {
    pub instance_id: InstanceId,
    pub type_id: TypeId,
    pub name: &'static str,
    pub value: Box<dyn AnyCloneable + Sync + Send>,
}

#[derive(Debug, Clone)]
pub(crate) struct StateUpdate {
    pub instance_id: InstanceId,
    pub index: usize,
    pub value: Box<dyn AnyCloneable + Send>,
}

#[derive(Debug)]
pub(crate) struct HooksEnvironment {
    pub(crate) set_states: Vec<StateUpdate>,
    /// Pending updates to contexts.
    ///
    /// This is modified through the returned `Setter` closure
    pub(crate) set_contexts: Vec<ContextUpdate>,
    pub(crate) frame_listeners: HashMap<InstanceId, Vec<FrameListener>>,
}
impl HooksEnvironment {
    pub(crate) fn new() -> Self {
        Self {
            set_states: Vec::new(),
            set_contexts: Vec::new(),
            frame_listeners: HashMap::new(),
        }
    }
    pub fn on_element_removed(&mut self, instance_id: &str) {
        self.frame_listeners.remove(instance_id);
    }
}
