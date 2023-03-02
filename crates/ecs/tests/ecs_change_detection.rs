use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use ambient_ecs::{components, query, query_mut, ArchetypeFilter, ComponentEntry, Entity, EntityId, FrameEvent, Query, QueryState, World};
use itertools::Itertools;

components!("test", {
    test: &'static str,
    test2: &'static str,
    a: f32,
    b: f32,
    c: f32,
    counter: usize,
});

fn init() {
    init_components();
}

#[test]
fn change_query() {
    init();
    let mut world = World::new("change_query");
    let e_a = world.spawn(Entity::new().with(a(), 1.));
    let q = Query::new(ArchetypeFilter::new().incl(a())).when_changed(a());
    let mut state = QueryState::new();

    assert_eq!(&[e_a], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);
    assert_eq!(&[] as &[EntityId], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    world.set(e_a, a(), 8.).unwrap();
    assert_eq!(&[e_a], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    world.set(e_a, a(), 2.).unwrap();
    world.set(e_a, a(), 2.).unwrap();
    assert_eq!(&[e_a], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    let e_b = world.spawn(Entity::new().with(a(), 1.));
    assert_eq!(&[e_b], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    world.set(e_a, a(), 2.).unwrap();
    world.despawn(e_a);
    assert_eq!(&[] as &[EntityId], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);
}

#[test]
fn change_query_dyn() {
    init();
    let mut world = World::new("change_query");
    let e_a = world.spawn(Entity::new().with(a(), 1.));
    let q = Query::new(ArchetypeFilter::new().incl(a())).when_changed(a());
    let mut state = QueryState::new();

    assert_eq!(&[e_a], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);
    assert_eq!(&[] as &[EntityId], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    world.set_entry(e_a, ComponentEntry::new(a(), 2.0)).unwrap();
    world.set_entry(e_a, ComponentEntry::new(a(), 2.0)).unwrap();
    assert_eq!(&[e_a], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    let e_b = world.spawn(Entity::new().with(a(), 1.));
    assert_eq!(&[e_b], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);

    world.set_entry(e_a, ComponentEntry::new(a(), 2.0)).unwrap();
    world.despawn(e_a);
    assert_eq!(&[] as &[EntityId], &q.iter(&world, Some(&mut state)).map(|x| x.id()).collect_vec()[..]);
}

#[test]
fn change_system() {
    init();
    let mut world = World::new("change_system");
    let counter = Arc::new(AtomicU32::new(0));
    let e_a = world.spawn(Entity::new().with(a(), 1.).with(b(), 2.));
    let mut a_from_b = {
        let counter = counter.clone();
        query_mut((a(),), (b().changed(),)).to_system(move |query, world: &mut World, state, _| {
            for (_, (a,), (&b,)) in query.iter(world, state) {
                *a = b;
                counter.fetch_add(1, Ordering::SeqCst);
            }
        })
    };

    a_from_b.run(&mut world, &FrameEvent);
    assert_eq!(world.get(e_a, a()).unwrap(), 2.);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    a_from_b.run(&mut world, &FrameEvent);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    world.set(e_a, b(), 9.).unwrap();
    a_from_b.run(&mut world, &FrameEvent);
    assert_eq!(world.get(e_a, a()).unwrap(), 9.);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn despawn_query() {
    init();
    let mut world = World::new("despawn_query");
    let x = world.spawn(Entity::new().with(test(), "a"));
    let mut qs = QueryState::new();
    assert_eq!(query((test(),)).despawned().iter(&world, Some(&mut qs)).count(), 0);
    world.despawn(x);
    let mut c = 0;
    for (id, (&test,)) in query((test(),)).despawned().iter(&world, Some(&mut qs)) {
        assert_eq!(id, x);
        assert_eq!(test, "a");
        c += 1;
    }
    assert_eq!(c, 1);
    assert_eq!(0, query((test().changed(),)).iter(&world, Some(&mut QueryState::new())).count());
}

#[test]
fn change_query_multi_frame() {
    init();
    let mut world = World::new("change_query_multi_frame");
    let e_a = world.spawn(Entity::new().with(a(), 1.));
    let mut state = QueryState::new();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 1);
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 0);
    world.next_frame();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 0);

    world.set(e_a, a(), 2.).unwrap();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 1);
    world.next_frame();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 0);
    world.next_frame();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 0);

    world.set(e_a, a(), 3.).unwrap();
    world.next_frame();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 1);
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut state)).count(), 0);
}

#[test]
fn add_component_events() {
    init();
    let mut world = World::new("add_component_events");
    let mut qs1 = QueryState::new();
    let mut qs2 = QueryState::new();
    let mut qs3 = QueryState::new();
    let mut qs4 = QueryState::new();
    let mut qs5 = QueryState::new();
    let mut qs6 = QueryState::new();

    let mut qs_1 = QueryState::new();
    let mut qs_2 = QueryState::new();
    let mut qs_3 = QueryState::new();
    let mut qs_4 = QueryState::new();
    let mut qs_5 = QueryState::new();
    let mut qs_6 = QueryState::new();

    // Run twice because the second time the archetypes will already exist
    for _ in 0..2 {
        // []
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [+{ a }]
        let x = world.spawn(Entity::new().with(a(), 0.));
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 1);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 1);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [{ a, +b }]
        world.add_component(x, b(), 1.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 1);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 1);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 1);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [{ a, -b }]
        world.remove_component(x, b()).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 1);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 1);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 1);

        // [{ a, +b }]
        world.add_component(x, b(), 1.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 1);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 1);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 1);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [{ -a, b }]
        world.remove_component(x, a()).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 1);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 1);

        // [-{ b }]
        world.despawn(x);
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 1);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [+{ a, b }]
        let x = world.spawn(Entity::new().with(a(), 0.).with(b(), 1.));
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 1);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 1);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 1);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 1);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [-{ a, b }]
        world.despawn(x);
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 1);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 1);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 1);

        // [+{ a }]
        // [{ a, +b }]
        let x = world.spawn(Entity::new().with(a(), 0.));
        world.add_component(x, b(), 1.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 1);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 1);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 1);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 1);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [{ a, +-b }]
        world.remove_component(x, b()).unwrap();
        world.add_component(x, b(), 1.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [-{ a, b }]
        world.despawn(x);
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 1);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 1);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 1);

        // [+{ a }]
        let x = world.spawn(Entity::new().with(a(), 0.));
        world.set(x, a(), 1.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 1);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 1);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [{ *a }]
        world.set(x, a(), 2.).unwrap();
        world.set(x, a(), 3.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 1);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [-{ a }]
        world.despawn(x);
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 1);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [+{ a, b, c }]
        let x = world.spawn(Entity::new().with(a(), 0.));
        world.add_component(x, b(), 1.).unwrap();
        world.add_component(x, c(), 1.).unwrap();
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 1);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 1);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 1);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 1);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

        // [-{ a, b, c }]
        world.despawn(x);
        assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
        assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
        assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
        assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
        assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
        assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

        assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
        assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 1);
        assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
        assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 1);
        assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
        assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 1);
    }
}

#[test]
fn add_component_events_data_before_query_state() {
    init();
    let mut world = World::new("add_component_events_data_before_query_state");

    // [+{ a }]
    let x = world.spawn(Entity::new().with(a(), 0.));

    let mut qs1 = QueryState::new();
    let mut qs2 = QueryState::new();
    let mut qs3 = QueryState::new();
    let mut qs4 = QueryState::new();
    let mut qs5 = QueryState::new();
    let mut qs6 = QueryState::new();

    let mut qs_1 = QueryState::new();
    let mut qs_2 = QueryState::new();
    let mut qs_3 = QueryState::new();
    let mut qs_4 = QueryState::new();
    let mut qs_5 = QueryState::new();
    let mut qs_6 = QueryState::new();

    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 1);
    assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 0);
    assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 0);
    assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 0);
    assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 0);
    assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 1);

    assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 1);
    assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
    assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 0);
    assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
    assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 0);
    assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);

    // [{ a, +b }]
    world.add_component(x, b(), 1.).unwrap();
    assert_eq!(query((a().changed(),)).iter(&world, Some(&mut qs1)).count(), 0);
    assert_eq!(query((b().changed(),)).iter(&world, Some(&mut qs2)).count(), 1);
    assert_eq!(query((a().changed(), b(),)).iter(&world, Some(&mut qs3)).count(), 1);
    assert_eq!(query((a(), b().changed(),)).iter(&world, Some(&mut qs4)).count(), 1);
    assert_eq!(query((a().changed(), b().changed(),)).iter(&world, Some(&mut qs5)).count(), 1);
    assert_eq!(query((a().changed(),)).excl(b()).iter(&world, Some(&mut qs6)).count(), 0);

    assert_eq!(query((a(),)).spawned().iter(&world, Some(&mut qs_1)).count(), 0);
    assert_eq!(query((a(),)).despawned().iter(&world, Some(&mut qs_2)).count(), 0);
    assert_eq!(query((b(),)).spawned().iter(&world, Some(&mut qs_3)).count(), 1);
    assert_eq!(query((b(),)).despawned().iter(&world, Some(&mut qs_4)).count(), 0);
    assert_eq!(query((a(), b(),)).spawned().iter(&world, Some(&mut qs_5)).count(), 1);
    assert_eq!(query((a(), b())).despawned().iter(&world, Some(&mut qs_6)).count(), 0);
}
