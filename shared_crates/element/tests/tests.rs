use std::{sync::Arc, time::Duration};

use ambient_core::hierarchy::children;
use ambient_element::{
    element_component, use_state, Element, ElementComponentExt, ElementTree, Hooks,
};
mod common;
use common::*;

#[test]
fn single_root_element() {
    #[element_component]
    fn Root(hooks: &mut Hooks) -> Element {
        *hooks.world.resource_mut(n_renders()) += 1;
        Element::new().init_default(prop_a()).with(prop_b(), 3)
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
    #[element_component]
    fn Root(_: &mut Hooks, with_super: bool) -> Element {
        if with_super {
            Child.into()
        } else {
            Element::new()
        }
    }

    #[element_component]
    fn Child(_: &mut Hooks) -> Element {
        Element::new()
    }

    let mut world = initialize();
    let mut tree = Root { with_super: true }.el().spawn_tree(&mut world);
    tree.update(&mut world);
    tree.migrate_root(&mut world, Root { with_super: false }.el());
    tree.update(&mut world);
}

#[test]
fn rerender_child() {
    #[element_component]
    fn Root(_: &mut Hooks, child_id: u32) -> Element {
        Element::new().children(vec![Child { _id: child_id }.into()])
    }

    #[element_component]
    fn Child(_: &mut Hooks, _id: u32) -> Element {
        Element::new()
    }

    let mut world = initialize();
    let start_n_entities = world.len();
    let mut tree = ElementTree::new(&mut world, Element::from(Root { child_id: 0 }));
    assert_eq!(2, world.len() - start_n_entities);
    assert_eq!(4, tree.n_instances());
    assert_eq!(
        1,
        world
            .get_cloned(tree.root_entity().unwrap(), children())
            .unwrap()
            .len()
    );

    tree.migrate_root(&mut world, Element::from(Root { child_id: 1 }));
    assert_eq!(2, world.len() - start_n_entities);
    assert_eq!(4, tree.n_instances());
}

#[test]
fn parent_components_should_stay_after_child_rerenders() {
    #[element_component]
    fn Root(_: &mut Hooks) -> Element {
        Child.el().with(prop_b(), 1).with(prop_a(), ())
    }

    #[element_component]
    fn Child(hooks: &mut Hooks) -> Element {
        let (state, set_state) = use_state(hooks, 9);
        hooks
            .world
            .add_component(
                hooks.world.resource_entity(),
                element_cb(),
                Arc::new(move || set_state(10)),
            )
            .unwrap();
        Element::new().with(prop_b(), state as u32)
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
    #[element_component]
    fn Root(hooks: &mut Hooks) -> Element {
        let (state, set_state) = use_state(hooks, 0);
        *hooks.world.resource_mut(n_renders()) += 1;
        Element::new().on_spawned(move |_, _, _| {
            let set_state = set_state.clone();
            tokio::task::spawn(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                set_state(state + 1);
            });
        })
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
    #[element_component]
    fn Root(_: &mut Hooks) -> Element {
        Element::new()
            .init_default(prop_a())
            .on_spawned(|world, ent, _| {
                // Should be possible to access components in on_spawned
                world.get(ent, prop_a()).unwrap();
            })
    }

    let mut world = initialize();
    let mut tree = Root.el().spawn_tree(&mut world);
    tree.update(&mut world);
}

#[test]
fn changing_roots() {
    #[element_component]
    fn Testy(_: &mut Hooks) -> Element {
        Element::new()
    }
    #[element_component]
    fn Testy2(_: &mut Hooks) -> Element {
        Element::new()
    }

    let mut world = initialize();
    let mut tree = Testy.el().spawn_tree(&mut world);
    tree.migrate_root(&mut world, Testy2.el());
    tree.migrate_root(&mut world, Element::new());
}
