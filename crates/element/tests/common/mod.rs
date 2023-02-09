use std::sync::Arc;

use elements_ecs::{components, query_mut, Resource, World};
use elements_std::events::EventDispatcher;
use itertools::Itertools;

components!("test", {
    prop_a: (),
    prop_b: u32,
    trigger: EventDispatcher<dyn Fn(&mut World) + Send + Sync>,
    @[Resource]
    counter: u32,
    @[Resource]
    n_renders: u32,
    element_cb: Arc<dyn Fn() + Send + Sync>,
});

pub fn initialize() -> World {
    elements_core::hierarchy::init_components();
    elements_element::init_components();
    init_components();

    let mut world = World::new("initialize");
    world.add_component(world.resource_entity(), counter(), 0).unwrap();
    world.add_component(world.resource_entity(), n_renders(), 0).unwrap();
    world
}

#[allow(dead_code)]
pub fn run_triggers(world: &mut World) {
    let triggers = query_mut((), (trigger(),)).iter(world, None).map(|x| x.2 .0.clone()).collect_vec();
    for trigger in triggers.into_iter() {
        for handler in trigger.iter() {
            handler(world);
        }
    }
}
