use std::ops::Bound;

use itertools::Itertools;
use kiwi_ecs::{
    components, index_system, ArchetypeFilter, EntityData, FrameEvent, Index, IndexColumns, IndexField, IndexKey, System, World
};

components!("test", {
    a: i32,
    test_index: Index,
});

fn init() {
    init_components();
}

#[test]
fn simple_index() {
    init();
    let mut world = World::new("simple_index");
    let mut index = Index::new(IndexColumns::new().add_column(a()));

    let x = world.spawn(EntityData::new().set(a(), 5));
    index.insert_entity(&world, x);

    let start = Bound::Included(IndexKey::min(vec![IndexField::exact(a(), 5)]));
    assert_eq!(index.range((start.clone(), Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x]);

    let y = world.spawn(EntityData::new().set(a(), 3));
    index.insert_entity(&world, y);
    assert_eq!(index.range((start.clone(), Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x]);
    let z = world.spawn(EntityData::new().set(a(), 7));
    index.insert_entity(&world, z);
    assert_eq!(index.range((start, Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x, z]);
}

#[test]
fn index_as_resource() {
    init();
    let mut world = World::new("index_as_resource");
    let mut systems = index_system(ArchetypeFilter::new(), IndexColumns::new().add_column(a()), test_index());
    let x = world.spawn(EntityData::new().set(a(), 5));
    systems.run(&mut world, &FrameEvent);
    let start = Bound::Included(IndexKey::min(vec![IndexField::exact(a(), 5)]));
    assert_eq!(world.resource(test_index()).range((start.clone(), Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x]);
    let _y = world.spawn(EntityData::new().set(a(), 3));
    systems.run(&mut world, &FrameEvent);
    assert_eq!(world.resource(test_index()).range((start.clone(), Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x]);
    let z = world.spawn(EntityData::new().set(a(), 7));
    systems.run(&mut world, &FrameEvent);
    assert_eq!(world.resource(test_index()).range((start, Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x, z]);
}

#[test]
fn changes() {
    init();
    let mut world = World::new("changes");
    let mut systems = index_system(ArchetypeFilter::new(), IndexColumns::new().add_column(a()), test_index());
    let x = world.spawn(EntityData::new().set(a(), 5));
    systems.run(&mut world, &FrameEvent);
    let start = Bound::Included(IndexKey::min(vec![IndexField::exact(a(), 5)]));
    assert_eq!(world.resource(test_index()).range((start.clone(), Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x]);

    world.set(x, a(), 3).unwrap();
    systems.run(&mut world, &FrameEvent);
    assert_eq!(world.resource(test_index()).range((start.clone(), Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![]);

    world.set(x, a(), 7).unwrap();
    systems.run(&mut world, &FrameEvent);
    assert_eq!(world.resource(test_index()).range((start, Bound::Unbounded)).map(|x| x.id().unwrap()).collect_vec(), vec![x]);
}

#[test]
fn simple_index_exact() {
    init();
    let mut world = World::new("simple_index_exact");
    let mut index = Index::new(IndexColumns::new().add_column(a()));

    let x = world.spawn(EntityData::new().set(a(), 5));
    index.insert_entity(&world, x);
    let dup = world.spawn(EntityData::new().set(a(), 5));
    index.insert_entity(&world, dup);

    let start = IndexKey::min(vec![IndexField::exact(a(), 5)]);
    let end = IndexKey::max(vec![IndexField::exact(a(), 5)]);
    assert_eq!(index.range(&start..&end).map(|x| x.id().unwrap()).sorted().collect_vec(), vec![x, dup].into_iter().sorted().collect_vec());

    let y = world.spawn(EntityData::new().set(a(), 3));
    index.insert_entity(&world, y);
    assert_eq!(index.range(&start..).map(|x| x.id().unwrap()).sorted().collect_vec(), vec![x, dup].into_iter().sorted().collect_vec());

    let z = world.spawn(EntityData::new().set(a(), 7));
    index.insert_entity(&world, z);
    assert_eq!(index.range(&start..).map(|x| x.id().unwrap()).sorted().collect_vec(), vec![x, dup, z].into_iter().sorted().collect_vec());
}

#[test]
fn multiple_for_exact_value() {
    init();
    let mut world = World::new("multiple_for_exact_value");
    let mut index = Index::new(IndexColumns::new().add_column(a()));

    let x = world.spawn(EntityData::new().set(a(), 3));
    let y = world.spawn(EntityData::new().set(a(), 5));
    let z = world.spawn(EntityData::new().set(a(), 5));
    let w = world.spawn(EntityData::new().set(a(), 7));
    index.insert_entity(&world, x);
    index.insert_entity(&world, y);
    index.insert_entity(&world, z);
    index.insert_entity(&world, w);

    let start = Bound::Included(IndexKey::min(vec![IndexField::exact(a(), 5)]));
    let end = Bound::Included(IndexKey::max(vec![IndexField::exact(a(), 5)]));
    assert_eq!(index.range((start, end)).map(|x| x.id().unwrap()).sorted().collect_vec(), vec![y, z].into_iter().sorted().collect_vec());
}
