use std::sync::Arc;

use ambient_ecs::{
    components, ArchetypeFilter, Entity, EntityId, FrozenWorldDiff, Query, Serializable, World,
    WorldDiff, WorldStream, WorldStreamFilter,
};
use itertools::Itertools;

components!("test", {
    @[Serializable]
    a: f32,
    b: f32,
    c: f32,
    no_sync: (),
});

fn init() {
    init_components();
}

#[test]
fn from_a_to_b_diff() {
    init();
    let mut from = World::new("from_a_to_b_diff");
    Entity::new().with(a(), 5.).with(b(), 2.).spawn(&mut from);
    let to = from.clone();
    let diff = WorldDiff::from_a_to_b(WorldStreamFilter::default(), &from, &to);
    assert_eq!(diff.changes.len(), 0);
}

#[test]
fn from_a_to_b_remove_component() {
    init();
    let mut from = World::new("from_a_to_b_remove_component");
    let x = Entity::new().with(a(), 5.).with(b(), 2.).spawn(&mut from);
    let y = Entity::new().with(a(), 5.).with(b(), 2.).spawn(&mut from);
    let mut to = from.clone();
    to.remove_component(x, b()).unwrap();
    to.remove_component(y, b()).unwrap();
    let diff = WorldDiff::from_a_to_b(WorldStreamFilter::default(), &from, &to);
    assert_eq!(diff.changes.len(), 2);
    for c in &diff.changes {
        c.is_remove_components();
    }
}

#[test]
fn streaming() {
    init();
    let mut source = World::new_with_config("streaming_src", true);
    source.init_shape_change_tracking();
    source
        .add_component(source.resource_entity(), no_sync(), ())
        .ok();
    let mut dest = World::new("streaming_dst");
    let mut stream = WorldStream::new(WorldStreamFilter::new(
        ArchetypeFilter::new().excl(no_sync()),
        Arc::new(|_, _| true),
    ));

    let x = Entity::new().with(a(), 1.).spawn(&mut source);
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.set(x, a(), 2.).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.add_component(x, b(), 9.).unwrap();
    let diff = stream.next_diff(&source);
    assert_eq!(diff.changes.iter().filter(|c| c.is_set()).count(), 1);
    diff.apply(&mut dest, Entity::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.remove_component(x, a()).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.despawn(x).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));
}

fn dump_content_string(world: &World) -> String {
    Query::all()
        .iter(world, None)
        .filter_map(|ea| {
            let id = ea.id();
            if id != world.resource_entity() {
                let data = world
                    .get_components(id)
                    .unwrap()
                    .into_iter()
                    .map(|desc| {
                        let value = world.get_entry(id, desc).unwrap();
                        format!("{:?}: {:?}", desc, desc.as_debug(&value))
                    })
                    .join(", ");
                Some(format!("[{id} {data}]"))
            } else {
                None
            }
        })
        .join(" ")
}

#[test]
fn serialization_of_worlddiff_variants() {
    // Arrange
    init();
    let some_entity = EntityId::new();
    let diff = WorldDiff::new()
        .add_component(some_entity, a(), 5.)
        .add_component(EntityId::new(), a(), 42.)
        .set(some_entity, a(), 10.0);
    let frozen_diff: FrozenWorldDiff = diff.clone().into();
    let frozen_diffs = vec![frozen_diff.clone()];
    let diff_view = FrozenWorldDiff::merge(&frozen_diffs);

    // Act
    let serialized = bincode::serialize(&diff).unwrap();
    let frozen_serialized = bincode::serialize(&frozen_diff).unwrap();
    let view_serialized = bincode::serialize(&diff_view).unwrap();

    // Assert
    assert_eq!(serialized, frozen_serialized);
    assert_eq!(serialized, view_serialized);
}
