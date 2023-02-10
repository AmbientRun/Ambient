use std::{sync::Arc, time::Duration};

use kiwi_core::hierarchy::children;
use kiwi_ecs::World;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, ElementTree, Hooks};
mod common;
use common::*;

#[test]
fn single_root_element() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, world: &mut World, _: &mut Hooks) -> Element {
            *world.resource_mut(n_renders()) += 1;
            Element::new().init_default(prop_a()).set(prop_b(), 3)
        }
    }

    let mut world = initialize();
    let start_n_entities = world.len();
    let mut tree = Root.el().spawn_tree(&mut world);
    assert_eq!(1, *world.resource(n_renders()));
    assert_eq!(1, world.len() - start_n_entities);

    tree.migrate_root(&mut world, Root.el());
    tree.update(&mut world);
    assert_eq!(2, *world.resource(n_renders()));
    assert_eq!(1, world.len() - start_n_entities);
}

#[test]
fn remove_super() {
    #[derive(Debug, Clone)]
    pub struct Root {
        with_super: bool,
    }
    impl ElementComponent for Root {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            if self.with_super {
                Child.into()
            } else {
                Element::new()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child;
    impl ElementComponent for Child {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            Element::new()
        }
    }

    let mut world = initialize();
    let mut tree = Root { with_super: true }.el().spawn_tree(&mut world);
    tree.update(&mut world);
    tree.migrate_root(&mut world, Root { with_super: false }.el());
    tree.update(&mut world);
}

#[test]
fn rerender_child() {
    #[derive(Debug, Clone)]
    pub struct Root {
        child_id: u32,
    }
    impl ElementComponent for Root {
        fn render(self: Box<Self>, _world: &mut World, _: &mut Hooks) -> Element {
            Element::new().children(vec![Child { _id: self.child_id }.into()])
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child {
        _id: u32,
    }
    impl ElementComponent for Child {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            Element::new()
        }
    }

    let mut world = initialize();
    let start_n_entities = world.len();
    let mut tree = ElementTree::new(&mut world, Element::from(Root { child_id: 0 }));
    assert_eq!(2, world.len() - start_n_entities);
    assert_eq!(4, tree.n_instances());
    assert_eq!(1, world.get_ref(tree.root_entity().unwrap(), children()).unwrap().len());

    tree.migrate_root(&mut world, Element::from(Root { child_id: 1 }));
    assert_eq!(2, world.len() - start_n_entities);
    assert_eq!(4, tree.n_instances());
}

#[test]
fn parent_components_should_stay_after_child_rerenders() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            Child.el().set(prop_b(), 1).set(prop_a(), ())
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child;
    impl ElementComponent for Child {
        fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(9);
            world.add_component(world.resource_entity(), element_cb(), Arc::new(move || set_state(10))).unwrap();
            Element::new().set(prop_b(), state as u32)
        }
    }

    let mut world = initialize();
    let mut tree = ElementTree::new(&mut world, Root.el());
    assert_eq!(world.get(tree.root_entity().unwrap(), prop_b()).unwrap(), 1);
    assert!(world.has_component(tree.root_entity().unwrap(), prop_a()));
    world.resource(element_cb())();
    tree.update(&mut world);
    assert_eq!(world.get(tree.root_entity().unwrap(), prop_b()).unwrap(), 1);
    assert!(world.has_component(tree.root_entity().unwrap(), prop_a()));
}

#[tokio::test]
async fn async_task() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(0);
            *world.resource_mut(n_renders()) += 1;
            Element::new().on_spawned(move |_, _| {
                let set_state = set_state.clone();
                tokio::task::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    set_state(state + 1);
                });
            })
        }
    }

    let mut world = initialize();
    let mut tree = Root.el().spawn_tree(&mut world);
    tree.update(&mut world);
    assert_eq!(1, *world.resource(n_renders()));
    tokio::time::sleep(Duration::from_millis(50)).await;
    tree.update(&mut world);
    assert_eq!(2, *world.resource(n_renders()));
}

#[test]
fn remove_element_renderer() {
    let mut world = initialize();
    let start_n_entities = world.len();
    let mut tree = Element::new().spawn_tree(&mut world);
    assert_eq!(1, world.len() - start_n_entities);
    tree.remove_root(&mut world);
    tree.update(&mut world);
    assert_eq!(0, world.len() - start_n_entities);
}

#[test]
fn on_spawned() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            Element::new().init_default(prop_a()).on_spawned(|world, ent| {
                // Should be possible to access components in on_spawned
                world.get(ent, prop_a()).unwrap();
            })
        }
    }

    let mut world = initialize();
    let mut tree = Root.el().spawn_tree(&mut world);
    tree.update(&mut world);
}

#[test]
fn changing_roots() {
    #[derive(Debug, Clone)]
    pub struct Testy;
    impl ElementComponent for Testy {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            Element::new()
        }
    }
    #[derive(Debug, Clone)]
    pub struct Testy2;
    impl ElementComponent for Testy2 {
        fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
            Element::new()
        }
    }

    let mut world = initialize();
    let mut tree = Testy.el().spawn_tree(&mut world);
    tree.migrate_root(&mut world, Testy2.el());
    tree.migrate_root(&mut world, Element::new());
}
