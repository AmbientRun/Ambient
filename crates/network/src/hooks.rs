use std::{collections::HashMap, sync::Arc};

use kiwi_core::runtime;
use kiwi_ecs::{
    query, ArchetypeFilter, Component, ComponentQuery, ComponentValue, ECSError, EntityId, FrameEvent, QueryState, TypedReadQuery, World,
    WorldDiff,
};
use kiwi_element::{Hooks, Setter};
use kiwi_std::{cb, Cb};

use crate::{client::GameClient, log_network_result, persistent_resources, player, rpc::rpc_world_diff, synced_resources, user_id};

pub fn use_remote_world_system<
    'a,
    R: ComponentQuery<'a> + Clone + 'static,
    F: Fn(&TypedReadQuery<R>, &mut World, Option<&mut QueryState>, &FrameEvent) + Send + Sync + 'static,
>(
    hooks: &mut Hooks,
    query: TypedReadQuery<R>,
    run: F,
) {
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let query_state = hooks.use_ref_with(QueryState::new);
    hooks.use_frame(move |_| {
        let mut game_state = game_client.game_state.lock();
        let mut qs = query_state.lock();
        run(&query, &mut game_state.world, Some(&mut qs), &FrameEvent);
    });
}

pub fn use_remote_component<T: ComponentValue + std::fmt::Debug>(
    hooks: &mut Hooks,
    entity: EntityId,
    component: Component<T>,
) -> Result<T, ECSError> {
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let component_version = hooks.use_ref_with(|| {
        let game_state = game_client.game_state.lock();
        game_state.world.get_component_content_version(entity, component.index()).ok()
    });
    let (value, set_value) = hooks.use_state_with(|| {
        let game_state = game_client.game_state.lock();
        game_state.world.get_ref(entity, component).cloned()
    });
    hooks.use_frame(move |_| {
        let game_state = game_client.game_state.lock();
        let mut cv = component_version.lock();
        let version = game_state.world.get_component_content_version(entity, component.index()).ok();
        if *cv != version {
            *cv = version;
            set_value(game_state.world.get_ref(entity, component).cloned());
        }
    });
    value
}

#[allow(clippy::type_complexity)]
pub fn use_remote_components<T: ComponentValue + std::fmt::Debug>(
    world: &mut World,
    hooks: &mut Hooks,
    arch_filter: ArchetypeFilter,
    component: Component<T>,
) -> Vec<(EntityId, T, Cb<dyn Fn(Option<T>) + Sync + Send>)> {
    let (values, set_values) = hooks.use_state(HashMap::new());
    let runtime = world.resource(runtime()).clone();

    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let qs_changed = hooks.use_ref_with(QueryState::new);
    let qs_despawned = hooks.use_ref_with(QueryState::new);
    let values_intermediate = hooks.use_ref_with(HashMap::new);
    hooks.use_frame(move |_| {
        let set_values = set_values.clone();
        let game_state = game_client.game_state.lock();
        let mut qs_changed = qs_changed.lock();
        let mut qs_despawned = qs_despawned.lock();
        let mut values = values_intermediate.lock();

        let mut changed = false;
        for (id, (value,)) in query((component.changed(),)).filter(&arch_filter).iter(&game_state.world, Some(&mut *qs_changed)) {
            let game_client = game_client.clone();
            let runtime = runtime.clone();
            let update: Cb<dyn Fn(Option<T>) + Sync + Send> = cb(move |value| {
                let game_client = game_client.clone();
                runtime.spawn(async move {
                    log_network_result!(
                        game_client
                            .rpc(
                                rpc_world_diff,
                                match value {
                                    Some(value) => WorldDiff::new().set(id, component, value),
                                    None => WorldDiff::new().remove_components_raw(id, vec![component.into()]),
                                },
                            )
                            .await
                    );
                });
            });
            values.insert(id, (id, value.clone(), update));
            changed = true;
        }
        for (id, _) in query((component,)).despawned().filter(&arch_filter).iter(&game_state.world, Some(&mut *qs_despawned)) {
            values.remove(&id);
            changed = true;
        }

        if changed {
            set_values(values.clone());
        }
    });

    values.into_values().collect()
}

#[allow(clippy::type_complexity)]
pub fn use_remote_first_component<T: ComponentValue + std::fmt::Debug>(
    hooks: &mut Hooks,
    world: &mut World,
    arch_filter: ArchetypeFilter,
    component: Component<T>,
) -> (Option<T>, Arc<dyn Fn(Option<T>) + Sync + Send>) {
    let (value, set_value) = hooks.use_state(None);
    let (entity_id, set_entity_id) = hooks.use_state(EntityId::null());
    use_remote_world_system(hooks, query((component.changed(),)).filter(&arch_filter), {
        let set_value = set_value.clone();
        let set_entity_id = set_entity_id.clone();
        move |q, world, qs, _| {
            if let Some((id, (value,))) = q.iter(world, qs).next() {
                set_value(Some(value.clone()));
                set_entity_id(id);
            }
        }
    });
    use_remote_world_system(hooks, query((component,)).despawned().filter(&arch_filter), move |q, world, qs, _| {
        if q.iter(world, qs).next().is_some() {
            set_value(None);
            set_entity_id(EntityId::null());
        }
    });
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let runtime = world.resource(runtime()).clone();
    (
        value,
        Arc::new(move |value| {
            let game_client = game_client.clone();
            runtime.spawn(async move {
                log_network_result!(
                    game_client
                        .rpc(
                            rpc_world_diff,
                            match value {
                                Some(value) => WorldDiff::new().set(entity_id, component, value),
                                None => WorldDiff::new().remove_components_raw(entity_id, vec![component.into()]),
                            },
                        )
                        .await
                );
            });
        }),
    )
}

#[allow(clippy::type_complexity)]
/// A **persistent** component shared with all clients
pub fn use_remote_persisted_resource<T: ComponentValue + std::fmt::Debug>(
    hooks: &mut Hooks,
    world: &mut World,
    component: Component<T>,
) -> (Option<T>, Arc<dyn Fn(Option<T>) + Sync + Send>) {
    use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(persistent_resources()), component)
}

#[allow(clippy::type_complexity)]
/// A **non** persistent component shared with all clients
pub fn use_remote_synced_resource<T: ComponentValue + std::fmt::Debug>(
    hooks: &mut Hooks,
    world: &mut World,
    component: Component<T>,
) -> (Option<T>, Arc<dyn Fn(Option<T>) + Sync + Send>) {
    use_remote_first_component(hooks, world, ArchetypeFilter::new().incl(synced_resources()), component)
}

pub fn use_player_id(hooks: &mut Hooks) -> Option<EntityId> {
    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let (ent, set_ent) = hooks.use_state(None);
    use_remote_world_system(hooks, query(user_id().changed()).incl(player()), move |q, world, qs, _| {
        for (id, pid) in q.iter(world, qs) {
            if pid == &game_client.user_id {
                set_ent(Some(id));
            }
        }
    });
    ent
}
pub fn use_remote_player_component<T: ComponentValue + Default + std::fmt::Debug + Clone>(
    world: &mut World,
    hooks: &mut Hooks,
    component: Component<T>,
) -> (T, Setter<T>) {
    let player_id = use_player_id(hooks);
    let (value, set_value) = hooks.use_state(T::default());
    use_remote_world_system(hooks, query((component.changed(),)), move |q, world, qs, _| {
        for (id, (value,)) in q.iter(world, qs) {
            if Some(id) == player_id {
                set_value(value.clone());
            }
        }
    });

    let (game_client, _) = hooks.consume_context::<GameClient>().unwrap();
    let runtime = world.resource(runtime()).clone();
    let set_value = cb(move |new_value| {
        let game_client = game_client.clone();
        runtime.spawn(async move {
            let diff = WorldDiff::new().set(player_id.unwrap(), component, new_value);
            log_network_result!(game_client.rpc(rpc_world_diff, diff).await);
        });
    });

    (value, set_value)
}
