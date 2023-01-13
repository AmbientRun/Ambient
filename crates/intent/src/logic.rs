use std::time::SystemTime;

use elements_ecs::{query, Component, ComponentValue, EntityData, EntityId, IndexField, IndexKey, World};
use elements_network::server::SharedServerState;

use crate::{
    intent, intent_applied, intent_id, intent_index, intent_index_applied, intent_index_reverted, intent_registry, intent_reverted, intent_timestamp, intent_user_id
};

fn despawn_reverted_intents(world: &mut World, user_id: &str) {
    for (id, u) in query(intent_user_id()).incl(intent_reverted()).collect_cloned(world, None) {
        if u == user_id {
            world.despawn(id);
        }
    }
}

/// Pushes and applied the intent
pub fn push_intent(state: SharedServerState, user_id: String, mut data: EntityData) -> elements_ecs::EntityId {
    let (reg, id, intent) = {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(&user_id).unwrap();

        // Make sure to remove the undone intents, to start with a fresh stack
        despawn_reverted_intents(world, &user_id);

        data.set_self(intent_user_id(), user_id.clone());
        let intent = data.get(intent()).expect("Missing intent kind for intent");

        let id = data.spawn(world);
        let reg = world.resource(intent_registry()).clone();
        (reg, id, intent)
    };

    reg.apply_intent(state, intent, &user_id, id);
    id
}

pub fn create_intent<T: ComponentValue>(intent_arg: Component<T>, arg: T, collapse_id: Option<String>) -> EntityData {
    EntityData::new()
        .set(intent(), intent_arg.index())
        .set(intent_timestamp(), SystemTime::now())
        .set(intent_arg, arg)
        .set(intent_id(), collapse_id.unwrap_or_else(friendly_id::create))
}

/// Reverts the head intent iff it is the specified intent
pub fn undo_head_exact(state: SharedServerState, user_id: &str, intent: &str) -> Option<EntityId> {
    let (reg, id, intent) = {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(user_id).unwrap();

        let id = match get_head_intent(world, user_id) {
            Some(id) => id,
            None => return None,
        };

        let intent_id = world.get_ref(id, intent_id()).unwrap();
        if intent != intent_id {
            return None;
        }

        tracing::info!("Reverting intent: {intent}");
        let reg = world.resource(intent_registry()).clone();
        let intent = world.get(id, super::intent()).expect("Not an intent");
        (reg, id, intent)
    };

    reg.revert_intent(state, intent, user_id, id);

    Some(id)
}
pub fn undo_head(state: SharedServerState, user_id: &str) -> Option<EntityId> {
    let (reg, id, intent) = {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(user_id).unwrap();

        if let Some(id) = get_head_intent(world, user_id) {
            let reg = world.resource(intent_registry()).clone();
            let intent = world.get(id, intent()).expect("Not an intent");

            Some((reg, id, intent))
        } else {
            tracing::warn!("No more intents to undo");
            None
        }
    }?;

    reg.revert_intent(state, intent, user_id, id);
    Some(id)
}

pub async fn redo_intent(state: SharedServerState, user_id: &str) -> Option<EntityId> {
    let (reg, id, intent) = {
        let mut guard = state.lock();
        let world = guard.get_player_world_mut(user_id).unwrap();
        let id = match get_tail_revert_intent(world, user_id) {
            Some(id) => id,
            _ => return None,
        };

        let intent = world.get(id, intent()).expect("Not an intent");

        world.remove_components(id, vec![intent_reverted().desc(), intent_applied().desc()]).unwrap();

        let reg = world.resource(intent_registry()).clone();
        (reg, id, intent)
    };

    reg.apply_intent(state, intent, user_id, id);

    Some(id)
}

// Internal

pub(crate) fn get_head_intent(world: &World, user_id: &str) -> Option<EntityId> {
    let start = IndexKey::min(vec![IndexField::exact(intent_user_id(), user_id.to_string()), IndexField::Min]);
    let end = IndexKey::max(vec![IndexField::exact(intent_user_id(), user_id.to_string()), IndexField::Max]);
    world.resource(intent_index()).range(start..end).last().map(|x| x.id().unwrap())
}

pub(crate) fn get_tail_revert_intent(world: &World, user_id: &str) -> Option<EntityId> {
    let start = IndexKey::min(vec![IndexField::exact(intent_user_id(), user_id.to_string()), IndexField::Min]);
    let end = IndexKey::max(vec![IndexField::exact(intent_user_id(), user_id.to_string()), IndexField::Max]);
    world.resource(intent_index_reverted()).range(start..end).next().map(|x| x.id().unwrap())
}

pub(crate) fn get_head_applied_intent(world: &World, user_id: &str) -> Option<EntityId> {
    let start = IndexKey::min(vec![IndexField::exact(intent_user_id(), user_id.to_string()), IndexField::Min]);
    let end = IndexKey::max(vec![IndexField::exact(intent_user_id(), user_id.to_string()), IndexField::Max]);
    let head = world.resource(intent_index_applied()).range(start..end).last().map(|x| x.id().unwrap());
    if let Some(head) = head {
        assert!(world.exists(head));
    }
    head
}
