use ambient_ecs::{
    components, copy_component_recursive,
    generated::hierarchy::components::{children, parent},
    Entity, FrameEvent, System, World,
};

components!("test", {
    hello: f32,
    hello_recursive: f32,
});

#[test]
fn test_copy_component_recursive() {
    init_components();
    ambient_ecs::init_components();
    let mut world = World::new_unknown("test");

    let c = Entity::new().spawn(&mut world);
    let b = Entity::new().with(parent(), c).spawn(&mut world);
    let a = Entity::new().with(parent(), b).spawn(&mut world);
    // no children
    let d = Entity::new().spawn(&mut world);

    world.add_component(a, children(), vec![b]).unwrap();
    world.add_component(b, children(), vec![c]).unwrap();
    world.add_component(a, hello_recursive(), 3.).unwrap();
    world.add_component(d, hello_recursive(), 4.).unwrap();

    let mut sys = copy_component_recursive("test", hello_recursive(), hello());
    sys.run(&mut world, &FrameEvent);
    assert_eq!(world.get(c, hello()).unwrap(), 3.);
    assert_eq!(world.get(d, hello()).unwrap(), 4.);

    world.next_frame();
    world.set(a, hello_recursive(), 2.).unwrap();
    world.set(d, hello_recursive(), 5.).unwrap();
    sys.run(&mut world, &FrameEvent);
    assert_eq!(world.get(c, hello()).unwrap(), 2.);
    assert_eq!(world.get(d, hello()).unwrap(), 5.);

    world.next_frame();
    world.remove_component(a, hello_recursive()).unwrap();
    world.remove_component(d, hello_recursive()).unwrap();
    sys.run(&mut world, &FrameEvent);
    assert!(!world.has_component(c, hello()));
    assert!(!world.has_component(d, hello()));
}
