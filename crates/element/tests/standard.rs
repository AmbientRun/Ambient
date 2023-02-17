use kiwi_ecs::query;
use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks, Memo};
mod common;
use common::*;

#[test]
fn memo() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner {
        state: u32,
    }
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            *hooks.world.resource_mut(n_renders()) += 1;
            Element::new()
        }
    }

    let mut world = initialize();
    let mut tree = Memo(Inner { state: 3 }).el().spawn_tree(&mut world);
    assert_eq!(1, *world.resource(n_renders()));

    for _ in 0..5 {
        tree.migrate_root(&mut world, Memo(Inner { state: 3 }).el());
        tree.update(&mut world);
        assert_eq!(1, *world.resource(n_renders()));
    }

    for _ in 0..5 {
        tree.migrate_root(&mut world, Memo(Inner { state: 6 }).el());
        tree.update(&mut world);
        assert_eq!(2, *world.resource(n_renders()));
    }
}

#[test]
fn memo_hook_state_update() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            *hooks.world.resource_mut(n_renders()) += 1;
            let (state, set_state) = hooks.use_state(0);
            Element::new().on_spawned(move |_, _| set_state(state + 1))
        }
    }

    let mut world = initialize();
    let mut tree = Memo(Inner).el().spawn_tree(&mut world);
    assert_eq!(1, *world.resource(n_renders()));
    tree.update(&mut world);
    assert_eq!(2, *world.resource(n_renders()));
}

#[test]
fn set_on_the_outside() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Test;
    impl ElementComponent for Test {
        fn render(self: Box<Self>, _: &mut Hooks) -> Element {
            Element::new().init(prop_b(), 5)
        }
    }

    let mut world = initialize();
    let el = Test.el();
    let mut tree = el.clone().spawn_tree(&mut world);
    assert_eq!(5, query(prop_b()).iter(&world, None).next().map(|(_, x)| *x).unwrap());
    tree.update(&mut world);

    let el = el.set(prop_b(), 7);
    tree.migrate_root(&mut world, el.clone());
    assert_eq!(7, query(prop_b()).iter(&world, None).next().map(|(_, x)| *x).unwrap());
    tree.update(&mut world);
    assert_eq!(7, query(prop_b()).iter(&world, None).next().map(|(_, x)| *x).unwrap());

    let el = el.set(prop_b(), 8);
    tree.migrate_root(&mut world, el);
    assert_eq!(8, query(prop_b()).iter(&world, None).next().map(|(_, x)| *x).unwrap());
    tree.update(&mut world);
    assert_eq!(8, query(prop_b()).iter(&world, None).next().map(|(_, x)| *x).unwrap());
}
