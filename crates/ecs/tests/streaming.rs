use std::sync::Arc;

use elements_ecs::{components, ArchetypeFilter, EntityData, Query, World, WorldDiff, WorldStream, WorldStreamFilter};
use itertools::Itertools;

components!("test", {
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
    EntityData::new().set(a(), 5.).set(b(), 2.).spawn(&mut from);
    let to = from.clone();
    let diff = WorldDiff::from_a_to_b(WorldStreamFilter::default(), &from, &to);
    assert_eq!(diff.changes.len(), 0);
}

#[test]
fn from_a_to_b_remove_component() {
    init();
    let mut from = World::new("from_a_to_b_remove_component");
    let x = EntityData::new().set(a(), 5.).set(b(), 2.).spawn(&mut from);
    let y = EntityData::new().set(a(), 5.).set(b(), 2.).spawn(&mut from);
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
    source.add_component(source.resource_entity(), no_sync(), ()).ok();
    let mut dest = World::new("streaming_dst");
    let mut stream = WorldStream::new(WorldStreamFilter::new(ArchetypeFilter::new().excl(no_sync()), Arc::new(|_, _| true)));

    let x = EntityData::new().set(a(), 1.).spawn(&mut source);
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, EntityData::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.set(x, a(), 2.).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, EntityData::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.add_component(x, b(), 9.).unwrap();
    let diff = stream.next_diff(&source);
    assert_eq!(diff.changes.iter().filter(|c| c.is_set()).count(), 0);
    diff.apply(&mut dest, EntityData::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.remove_component(x, a()).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, EntityData::new(), true);
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.despawn(x).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, EntityData::new(), true);
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
