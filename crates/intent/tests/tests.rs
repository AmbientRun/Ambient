use std::{collections::BTreeMap, sync::Arc};

use anyhow::bail;
use elements_ecs::{components, query, query_mut, EntityData, EntityId, FrameEvent, SimpleComponentRegistry, System, World};
use elements_intent::{
    common_intent_systems, intent_registry, logic::{create_intent, push_intent, redo_intent, undo_head}, use_old_state, IntentRegistry
};
use elements_network::server::{Player, ServerState, SharedServerState, MAIN_INSTANCE_ID};
use itertools::Itertools;
use parking_lot::Mutex;
use pretty_assertions::assert_eq;

components!("intent", {
    intent_add: f32,
    intent_add_undo: Vec<(EntityId, f32)>,
    intent_mul: f32,
    intent_mul_undo: Vec<(EntityId, f32)>,
    intent_fail: (),
    intent_fail_undo: (),

    value: f32,
});

fn create_test_entities(state: &Mutex<ServerState>, user_id: &str) -> BTreeMap<EntityId, f32> {
    let mut guard = state.lock();
    let world = guard.get_player_world_mut(user_id).unwrap();
    let values = [1.0, 2.0, 3.0];

    values
        .into_iter()
        .map(|v| {
            let id = EntityData::new().set(value(), v).spawn(world);
            (id, v)
        })
        .collect()
}

fn as_map(world: &World) -> BTreeMap<EntityId, f32> {
    query(value()).iter(world, None).map(|(id, v)| (id, *v)).collect()
}

fn register_intents(reg: &mut IntentRegistry) {
    reg.register(
        intent_add(),
        intent_add_undo(),
        |ctx, arg| {
            let world = ctx.world;
            Ok(query_mut(value(), ())
                .iter(world, None)
                .map(|(id, v, _)| {
                    let old_value = *v;

                    *v += arg;
                    (id, old_value)
                })
                .collect_vec())
        },
        |ctx, state| {
            let world = ctx.world;
            for (id, v) in state {
                world.add_component(id, value(), v).unwrap()
            }

            Ok(())
        },
        |old_arg, old_state, new_arg, _| (old_arg + new_arg, old_state.clone()),
    );

    reg.register(
        intent_mul(),
        intent_mul_undo(),
        |ctx, arg| {
            let world = ctx.world;
            Ok(query_mut(value(), ())
                .iter(world, None)
                .map(|(id, v, _)| {
                    let old_value = *v;

                    *v *= arg;
                    (id, old_value)
                })
                .collect_vec())
        },
        |ctx, state| {
            let world = ctx.world;
            for (id, v) in state {
                world.add_component(id, value(), v).unwrap()
            }

            Ok(())
        },
        |old_arg, old_state, new_arg, _| (old_arg * new_arg, old_state.clone()),
    );

    reg.register(
        intent_fail(),
        intent_fail_undo(),
        |_, ()| bail!("I told ya so"),
        |_, ()| panic!("You bafoon, how are undoing an intent which could not be applied in the first place"),
        use_old_state,
    )
}

fn setup_state() -> SharedServerState {
    let mut state = ServerState::new_local();

    let user_id = "user1".to_string();
    state.players.insert(user_id.clone(), Player::new_local(MAIN_INSTANCE_ID.to_string()));
    (common_intent_systems()).run(state.get_player_world_mut(&user_id).unwrap(), &FrameEvent);
    Arc::new(Mutex::new(state))
}

#[tokio::test]
async fn simple() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();
    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, None));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        eprintln!("World after push_intent: {:#?}", world.debug_archetypes());

        values.values_mut().for_each(|v| *v += 2.0);

        assert_eq!(values, as_map(world));
    }

    undo_head(state.clone(), &user_id);

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        eprintln!("World after undo_head: {:#?}", world.debug_archetypes());

        values.values_mut().for_each(|v| *v -= 2.0);

        assert_eq!(values, as_map(world));
    }
}

#[tokio::test]
async fn enqueued() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, None));
    push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 0.5, None));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v += 2.5);

        assert_eq!(values, as_map(world));
    }

    undo_head(state.clone(), &user_id);

    {
        values.values_mut().for_each(|v| *v -= 0.5);
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert_eq!(values, as_map(world));
    }
}

#[tokio::test]
async fn enqueued_collapse() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    let collapse_id = friendly_id::create();

    let x = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, Some(collapse_id.clone())));
    let y = push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 0.5, None));
    let z = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, Some(collapse_id.clone())));

    values.values_mut().for_each(|v| *v = (*v + 2.0) * 0.5 + 2.0);

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert!(world.exists(x));
        assert!(world.exists(y));
        assert!(world.exists(z));

        assert_eq!(values, as_map(world));
    }

    undo_head(state.clone(), &user_id);

    values.values_mut().for_each(|v| *v -= 2.0);

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert_eq!(values, as_map(world));
    }

    assert_eq!(undo_head(state.clone(), &user_id), Some(y));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v /= 0.5);
        assert_eq!(values, as_map(world));

        assert!(world.exists(x));
        assert!(world.exists(y));
        assert!(world.exists(z));

        dbg!(x, y, z);
    }
    let w = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), -4.0, Some(collapse_id)));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert!(!world.exists(x));
        assert!(!world.exists(y));
        assert!(!world.exists(z));
        assert!(world.exists(w));

        values.values_mut().for_each(|v| *v -= 4.0);

        assert_eq!(values, as_map(world));
    }
}

#[tokio::test]
async fn enqueue2() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, None));
    push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 0.5, None));
    push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, None));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v = (*v + 2.0) * 0.5 + 2.0);

        assert_eq!(values, as_map(world));
    }

    undo_head(state.clone(), &user_id);

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();

        values.values_mut().for_each(|v| *v -= 2.0);

        assert_eq!(values, as_map(world));
    }
}

#[tokio::test]
async fn enqueue2_redo() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    let collapse_id = friendly_id::create();

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    let x = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, Some(collapse_id.clone())));
    push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 0.5, None));
    let z = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, Some(collapse_id)));

    values.values_mut().for_each(|v| *v = (*v + 2.0) * 0.5 + 2.0);

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert_eq!(values, as_map(world));
    }

    assert_eq!(undo_head(state.clone(), &user_id), Some(z));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert!(world.exists(x));

        values.values_mut().for_each(|v| *v -= 2.0);

        assert_eq!(values, as_map(world));
    }

    assert_eq!(redo_intent(state.clone(), &user_id).await, Some(z));

    values.values_mut().for_each(|v| *v += 2.0);

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert_eq!(values, as_map(world));
    }
}

#[test]
fn undo_failed() {
    init_components();
    elements_intent::init_components();

    let user_id = "user1".to_string();
    let state = setup_state();
    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    let a = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 1.0, None));
    let b = push_intent(state.clone(), user_id.clone(), create_intent(intent_fail(), (), None));
    let c = push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 3.0, None));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v = (*v + 1.0) * 3.0);
        assert_eq!(values, as_map(world));
    }

    undo_head(state.clone(), &user_id);
    undo_head(state.clone(), &user_id);

    let _d = push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 2.0, None));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v = *v / 3.0 * 2.0);
        assert_eq!(values, as_map(world));

        assert!(world.exists(a));
        assert!(!world.exists(b));
        assert!(!world.exists(c));
    }
}

#[tokio::test]
async fn undo_push() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    let collapse_id = friendly_id::create();

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    let x = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 1.0, Some(collapse_id.clone())));

    let y = push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 5.0, Some(collapse_id.clone())));

    let z = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 0.5, Some(collapse_id.clone())));
    let w = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 0.5, Some(collapse_id.clone())));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v = (*v + 1.0) * 5.0 + 0.5 + 0.5);
        assert_eq!(values, as_map(world));

        assert!(!world.exists(z));
    }

    assert_eq!(undo_head(state.clone(), &user_id), Some(w));
    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();

        assert!(world.exists(w));
        values.values_mut().for_each(|v| *v -= 1.0);
        assert_eq!(values, as_map(world));
    }

    let a = push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 0.5, Some(collapse_id)));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert!(world.exists(x));
        // collapsed into a
        assert!(!world.exists(y));
        assert!(!world.exists(z));
        assert!(!world.exists(w));
        assert!(world.exists(a));

        values.values_mut().for_each(|v| *v *= 0.5);
        assert_eq!(values, as_map(world));
    }
}

#[tokio::test]
async fn redo_collapsed() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    register_intents(&mut reg);
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    let collapse_id = friendly_id::create();

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);

    let x = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, Some(collapse_id.clone())));
    let y = push_intent(state.clone(), user_id.clone(), create_intent(intent_mul(), 0.5, Some(collapse_id.clone())));
    let z = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 1.0, Some(collapse_id.clone())));
    let w = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 1.0, Some(collapse_id)));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        values.values_mut().for_each(|v| *v = (*v + 2.0) * 0.5 + 2.0);

        assert_eq!(values, as_map(world));
    }

    // z collapsed into w
    assert_eq!(undo_head(state.clone(), &user_id), Some(w));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();
        assert!(world.exists(x));

        values.values_mut().for_each(|v| *v -= 2.0);

        assert_eq!(values, as_map(world));
    }

    assert_eq!(redo_intent(state.clone(), &user_id).await, Some(w));

    {
        let guard = state.lock();
        let world = guard.get_player_world(&user_id).unwrap();

        values.values_mut().for_each(|v| *v += 2.0);

        assert_eq!(values, as_map(world));

        assert!(world.exists(x));
        assert!(world.exists(y));
        assert!(!world.exists(z));
        assert!(world.exists(w));
    }
}

#[tokio::test]
async fn collapse() {
    init_components();
    elements_intent::init_components();

    let state = setup_state();

    let user_id = "user1".to_string();

    let mut reg = IntentRegistry::new();
    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        register_intents(&mut reg);
        world.add_resource(intent_registry(), Arc::new(reg));
    }

    let collapse_id = friendly_id::create();

    // Create test entities
    let mut values = create_test_entities(&state, &user_id);
    let x = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 2.0, Some(collapse_id.clone())));
    let y = push_intent(state.clone(), user_id.clone(), create_intent(intent_add(), 1.0, Some(collapse_id)));

    {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();
        values.values_mut().for_each(|v| *v = (*v + 2.0) + 1.0);

        assert_eq!(values, as_map(world));

        // x is collapsed, and thus despawned
        assert!(!world.exists(x));
        assert!(world.exists(y));
    }
}
