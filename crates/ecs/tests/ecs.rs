use itertools::Itertools;
use kiwi_ecs::{components, query, query_mut, ECSError, EntityData, EntityId, Query, QueryState, Resource, World};

components!("test", {
    test: &'static str,
    test2: &'static str,
    a: f32,
    b: f32,
    c: f32,
    counter: usize,
    @[Resource]
    a_resource: (),
});

fn init() {
    init_components();
}

#[test]
#[should_panic]
fn unsound() {
    init();
    let mut world = World::new("unsound");
    let query = query_mut(a(), a());

    world.spawn(EntityData::new().set(a(), 5.0));
    for (_, unique, shared) in query.iter(&mut world, None) {
        let old = *shared;
        *unique = 42.0;
        // This should never happen as `shared` is a immutable reference.
        // `unique` does not abide aliasing rules and the compiler is free to
        // optimize the second read to shared away effectively accessing old no
        // longer valid memory. The compiler is free to optimize it to a no-op
        // and an assign since old and shared are the same.
        assert_ne!(*shared, old);
    }
}

#[test]
fn remove() {
    init();
    let mut world = World::new("remove");
    let a = world.spawn(EntityData::new().set(test(), "a"));
    let b = world.spawn(EntityData::new().set(test2(), "b"));
    world.despawn(a);
    world.despawn(b);
}

#[test]
fn iter_gap() {
    init();
    let mut world = World::new("iter_gap");
    let _a = world.spawn(EntityData::new().set(test(), "a"));
    let b = world.spawn(EntityData::new().set(test(), "b"));
    let _c = world.spawn(EntityData::new().set(test(), "c"));
    world.despawn(b);
    let entities = query((test(),)).iter(&world, None).map(|(_, (test,))| *test).collect_vec();
    assert_eq!(&["a", "c"], &entities[..]);
}

#[test]
fn add_component() {
    init();
    let mut world = World::new("add_component");
    let x = world.spawn(EntityData::new().set(a(), 0.));
    world.add_component(x, b(), 1.).unwrap();
    assert_eq!(1., world.get(x, b()).unwrap());
    let a_changed = query((a().changed(),)).iter(&world, Some(&mut QueryState::new())).map(|(id, _)| id).collect_vec();
    assert_eq!(&[x] as &[EntityId], &a_changed[..]);
    let b_changed = query((b().changed(),)).iter(&world, Some(&mut QueryState::new())).map(|(id, _)| id).collect_vec();
    assert_eq!(&[x], &b_changed[..]);
}

#[test]
fn remove_component() {
    init();
    let mut world = World::new("remove_component");
    let x = world.spawn(EntityData::new().set(a(), 0.).set(b(), 0.));
    assert_eq!(world.get_components(x).unwrap().len(), 2);
    world.remove_component(x, a()).unwrap();
    assert_eq!(world.get_components(x).unwrap().len(), 1);
    world.remove_component(x, b()).unwrap();
    assert_eq!(world.get_components(x).unwrap().len(), 0);
}

#[test]
fn spawn_all_excl_query() {
    init();
    let mut world = World::new("spawn_all_excl_query");
    let mut qs = QueryState::new();
    let q = Query::all().excl(a()).spawned();
    assert_eq!(q.iter(&world, Some(&mut qs)).count(), 1); // resources
    assert_eq!(q.iter(&world, Some(&mut qs)).count(), 0);
    let _x = world.spawn(EntityData::new().set(b(), 2.));
    assert_eq!(q.iter(&world, Some(&mut qs)).count(), 1);
    assert_eq!(q.iter(&world, Some(&mut qs)).count(), 0);
    world.spawn(EntityData::new().set(a(), 1.));
    assert_eq!(q.iter(&world, Some(&mut qs)).count(), 0);
}

#[test]
fn query_created_late() {
    init();
    let mut world = World::new("query_create_late");
    let _e_a = world.spawn(EntityData::new().set(a(), 1.));
    // Simulation runs for a while first
    for _ in 0..500 {
        world.next_frame();
    }
    let mut qs_change = QueryState::new();
    let mut qs_spawn = QueryState::new();
    assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_spawn)).count(), 1);
    assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_spawn)).count(), 0);
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs_change)).count(), 1);
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs_change)).count(), 0);
}

#[test]
fn remove_despawn() {
    init();
    let mut world = World::new("remove_despawn");
    let x = world.spawn(EntityData::new().set(a(), 0.));
    let y = world.spawn(EntityData::new().set(a(), 0.));
    world.remove_component(x, a()).unwrap();
    world.despawn(y);
}

#[test]
fn mirroring() {
    init();
    let mut world = World::new("mirroring");
    let id1 = EntityId(5);
    let id1b = EntityId(7);
    let id2 = EntityId(9);
    world.spawn_with_id(id1, EntityData::new().set(a(), 3.));
    world.get(id1, a()).unwrap();
    assert_eq!(query((a(),)).iter(&world, None).count(), 1);
    world.spawn_with_id(id2, EntityData::new().set(a(), 2.));
    world.get(id2, a()).unwrap();
    assert_eq!(query((a(),)).iter(&world, None).count(), 2);
    world.despawn(id1);
    assert_eq!(query((a(),)).iter(&world, None).count(), 1);
    world.spawn_with_id(id1b, EntityData::new().set(a(), 3.));
    assert_eq!(query((a(),)).iter(&world, None).count(), 2);
    world.get(id2, a()).unwrap();
    world.despawn(id2);
    assert_eq!(query((a(),)).iter(&world, None).count(), 1);
}

#[test]
fn content_version_should_remain_on_remove() {
    init();
    let mut world = World::new("content_version_should_remain_one");
    let x = EntityData::new().set(a(), 5.).set(b(), 2.).spawn(&mut world);
    let y = EntityData::new().set(a(), 5.).set(b(), 2.).spawn(&mut world);
    let x_start = world.get_component_content_version(x, a().index()).unwrap();
    let y_start = world.get_component_content_version(y, a().index()).unwrap();
    world.remove_component(x, b()).unwrap();
    assert_eq!(x_start, world.get_component_content_version(x, a().index()).unwrap());
    assert_eq!(y_start, world.get_component_content_version(y, a().index()).unwrap());
    world.remove_component(y, b()).unwrap();
    assert_eq!(x_start, world.get_component_content_version(x, a().index()).unwrap());
    assert_eq!(y_start, world.get_component_content_version(y, a().index()).unwrap());
}

#[test]
fn no_resources() {
    init();
    let world = World::new_with_config("no_resources", false);
    assert!(!world.exists(world.resource_entity()));
    assert!(world.resource_opt(a()).is_none());
}

#[test]
fn fresh_moveout_event_reader_should_work() {
    // Previously, moveout readers were being initialized with a fresh FramedEventsReader
    // with a frame of 0, resulting in panics when you attempted to read from them without
    // any events having occurred.
    // This test checks that this is no longer the case.

    init();
    let mut world = World::new_with_config("fresh_moveout_event_reader_should_work", false);

    // Ensure that spawn queries work correctly.
    let mut spawn_query_state = QueryState::new();
    let spawn_query = query(a()).spawned();
    assert_eq!(spawn_query.iter(&world, Some(&mut spawn_query_state)).count(), 0);

    let id = EntityData::new().set(a(), 5.).set(b(), 2.).spawn(&mut world);

    assert_eq!(spawn_query.iter(&world, Some(&mut spawn_query_state)).count(), 1);

    // Simulate `HISTORY_SIZE` number of frames to ensure that start_frame is incremented
    // for each archetype's moveout events, such that when the query runs, start_frame
    // is not zero
    for _ in 0..kiwi_ecs::FramedEvents::<()>::HISTORY_SIZE {
        world.next_frame();
    }

    // Check that the despawn query executes without panicking.
    let mut despawn_query_state = QueryState::new();
    let despawn_query = query(a()).despawned();
    assert_eq!(despawn_query.iter(&world, Some(&mut despawn_query_state)).count(), 0);

    world.despawn(id);
    assert_eq!(despawn_query.iter(&world, Some(&mut despawn_query_state)).count(), 1);
}

#[test]
fn errors_on_adding_a_resource_to_an_entity() {
    init();
    let mut world = World::new("errors_on_adding_a_resource_to_an_entity");
    let entity_id = world.spawn(EntityData::new());
    assert_eq!(
        world.add_component(entity_id, a_resource(), ()),
        Err(ECSError::AddedResourceToEntity { component_path: "core::test::a_resource".to_string(), entity_id })
    );
}

#[test]
fn can_add_a_resource() {
    init();
    World::new("can_add_a_resource").add_resource(a_resource(), ());
}
