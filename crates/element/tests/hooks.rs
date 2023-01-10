use std::sync::{
    atomic::{AtomicU32, Ordering}, Arc
};

use elements_ecs::World;
use elements_element::{Element, ElementComponent, ElementComponentExt, ElementTree, Hooks};
mod common;
use common::*;

#[test]
fn use_state() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
            *world.resource_mut(n_renders()) += 1;
            let (state, set_state) = hooks.use_state(0);
            Element::new().on_spawned(move |_, _| set_state(state + 1))
        }
    }

    let mut world = initialize();
    let mut tree = Inner.el().spawn_tree(&mut world);
    assert_eq!(1, *world.resource(n_renders()));
    tree.update(&mut world);
    assert_eq!(2, *world.resource(n_renders()));
    tree.update(&mut world);
    assert_eq!(2, *world.resource(n_renders()));
}

#[test]
fn use_state_inc_should_work() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(10);
            *world.resource_mut(n_renders()) += 1;
            *world.resource_mut(counter()) = state;
            Element::new().listener(trigger(), Arc::new(move |_| set_state(state + 1)))
        }
    }

    let mut world = initialize();
    *world.resource_mut(counter()) = 0;
    let mut tree = Inner.el().spawn_tree(&mut world);
    assert_eq!(*world.resource(counter()), 10);
    assert_eq!(*world.resource(n_renders()), 1);
    for i in 0..3 {
        run_triggers(&mut world);
        tree.update(&mut world);
        assert_eq!(*world.resource(counter()), 11 + i);
        assert_eq!(*world.resource(n_renders()), 2 + i);
    }
}

#[test]
fn use_state_on_removed_element_should_be_ignored() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(0);
            Element::new().listener(trigger(), Arc::new(move |_| set_state(state + 1)))
        }
    }

    let mut world = initialize();
    let mut tree = ElementTree::new(&mut world, Element::from(Inner));
    run_triggers(&mut world);
    tree.remove_root(&mut world);
    tree.update(&mut world);
}

#[test]
fn use_state_ensure_dropped() {
    static DROP_COUNT: AtomicU32 = AtomicU32::new(0);

    #[derive(Debug)]
    struct Droppable;
    impl Drop for Droppable {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
            let (_state, set_state) = hooks.use_state(None);
            Element::new().on_spawned(move |_, _| set_state(Some(Arc::new(Droppable))))
        }
    }
    let mut world = initialize();
    let mut tree = ElementTree::new(&mut world, Element::from(Inner));
    tree.update(&mut world);
    tree.remove_root(&mut world);
    assert_eq!(tree.n_instances(), 0);
    assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
}

#[test]
fn use_state_shouldnt_survive_between_instances() {
    static STATE: AtomicU32 = AtomicU32::new(0);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Test;
    impl ElementComponent for Test {
        fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(0);
            STATE.store(state, Ordering::Relaxed);
            Element::new().listener(trigger(), Arc::new(move |_| set_state(state + 1)))
        }
    }
    let mut world = initialize();
    let mut tree = Test.el().spawn_tree(&mut world);
    tree.update(&mut world);
    assert_eq!(STATE.load(Ordering::Relaxed), 0);

    run_triggers(&mut world);
    tree.update(&mut world);
    assert_eq!(STATE.load(Ordering::Relaxed), 1);

    tree.migrate_root(&mut world, Element::new());
    tree.update(&mut world);
    assert_eq!(STATE.load(Ordering::Relaxed), 1);

    tree.migrate_root(&mut world, Test.el());
    tree.update(&mut world);
    assert_eq!(STATE.load(Ordering::Relaxed), 0);
}

#[test]
fn use_memo() {
    #[derive(Debug, Clone)]
    pub struct Test {
        deps: u32,
    }
    impl ElementComponent for Test {
        fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
            hooks.use_memo_with(self.deps, |_| {
                *world.resource_mut(counter()) += 1;
                "test"
            });
            Element::new()
        }
    }

    let mut world = initialize();
    let mut tree = Test { deps: 0 }.el().spawn_tree(&mut world);
    assert_eq!(1, *world.resource(counter()));

    for _ in 0..3 {
        tree.migrate_root(&mut world, Test { deps: 0 }.el());
        tree.update(&mut world);
        assert_eq!(1, *world.resource(counter()));
    }

    for _ in 0..3 {
        tree.migrate_root(&mut world, Test { deps: 1 }.el());
        tree.update(&mut world);
        assert_eq!(2, *world.resource(counter()));
    }
}

#[test]
fn use_effect() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner {
        value: String,
    }
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
            hooks.use_effect(world, self.value, |world, _| {
                *world.resource_mut(counter()) += 1;
                *world.resource_mut(n_renders()) += 1;
                Box::new(|world| {
                    *world.resource_mut(counter()) -= 1;
                })
            });
            Element::new()
        }
    }

    let mut world = initialize();
    let mut tree = Inner { value: "Test".to_string() }.el().spawn_tree(&mut world);
    tree.update(&mut world);
    assert_eq!(1, *world.resource(counter()));
    assert_eq!(1, *world.resource(n_renders()));

    tree.migrate_root(&mut world, Inner { value: "Test".to_string() }.el());
    tree.update(&mut world);
    assert_eq!(1, *world.resource(counter()));
    assert_eq!(1, *world.resource(n_renders()));

    tree.migrate_root(&mut world, Inner { value: "Hello".to_string() }.el());
    tree.update(&mut world);
    assert_eq!(1, *world.resource(counter()));
    assert_eq!(2, *world.resource(n_renders()));

    tree.migrate_root(&mut world, Element::new());
    tree.update(&mut world);
    assert_eq!(0, *world.resource(counter()));
    assert_eq!(2, *world.resource(n_renders()));
}
