use ambient_ecs::query_mut;
use ambient_element::{Element, ElementComponent, ElementComponentExt, Hooks};
mod common;
use ambient_std::cb;
use common::*;

#[test]
fn test_outer_init() {
    #[derive(Debug, Clone)]
    pub struct Dummy;
    impl ElementComponent for Dummy {
        fn render(self: Box<Self>, _: &mut Hooks) -> Element {
            Element::new()
        }
    }

    #[derive(Debug, Clone)]
    pub struct Outer;
    impl ElementComponent for Outer {
        fn render(self: Box<Self>, _: &mut Hooks) -> Element {
            Element::from(Inner).init_default(prop_a())
        }
    }

    #[derive(Debug, Clone)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (count, set_count) = hooks.use_state(0);
            if count < 2 {
                Element::new().set(trigger(), cb(move |_| set_count(count + 1)))
            } else {
                Dummy.into()
            }
        }
    }

    let mut world = initialize();
    let mut tree = Outer.el().spawn_tree(&mut world);

    run_triggers(&mut world);
    tree.update(&mut world);
    assert_eq!(1, query_mut((), (prop_a(),)).iter(&mut world, None).count());

    run_triggers(&mut world);
    tree.update(&mut world);
    assert_eq!(1, query_mut((), (prop_a(),)).iter(&mut world, None).count());
}

#[test]
fn test_two_event_listeners() {
    #[derive(Debug, Clone)]
    pub struct Outer;
    impl ElementComponent for Outer {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (use_inner, set_use_inner) = hooks.use_state(true);
            if use_inner {
                Element::from(Inner).set(
                    trigger(),
                    cb(move |world| {
                        *world.resource_mut(counter()) += 1;
                        set_use_inner(false);
                    }),
                )
            } else {
                Element::new().set(
                    trigger(),
                    cb(move |world| {
                        *world.resource_mut(counter()) += 1;
                    }),
                )
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Inner;
    impl ElementComponent for Inner {
        fn render(self: Box<Self>, _: &mut Hooks) -> Element {
            Element::new().set(
                trigger(),
                cb(move |world| {
                    *world.resource_mut(counter()) += 1;
                }),
            )
        }
    }

    let mut world = initialize();
    let mut tree = Outer.el().spawn_tree(&mut world);

    run_triggers(&mut world);
    tree.update(&mut world);
    assert_eq!(2, *world.resource(counter()));

    run_triggers(&mut world);
    tree.update(&mut world);
    assert_eq!(3, *world.resource(counter()));
}

#[test]
fn update_state_on_replaced_element() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(0);
            Element::new().set(
                trigger(),
                cb(move |_| {
                    set_state(state + 1);
                }),
            )
        }
    }

    let mut world = initialize();
    let mut tree = Root.el().spawn_tree(&mut world);

    for _ in 0..3 {
        // The update is queued here, but then the Element is replaced, but this should still work because
        // the state should migrate to the new Element, as they are the same type
        run_triggers(&mut world);
        tree.migrate_root(&mut world, Element::from(Root));
        tree.update(&mut world);
    }
}

#[test]
fn update_state_on_root_and_child_simultaneously() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(0);
            Element::new()
                .set(
                    trigger(),
                    cb(move |_| {
                        set_state(state + 1);
                    }),
                )
                .children(vec![Child.into()])
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child;
    impl ElementComponent for Child {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state(0);
            *hooks.world.resource_mut(counter()) = state;
            Element::new().set(
                trigger(),
                cb(move |_| {
                    set_state(state + 1);
                }),
            )
        }
    }

    let mut world = initialize();
    let mut tree = Root.el().spawn_tree(&mut world);

    run_triggers(&mut world);
    tree.update(&mut world);
    assert_eq!(*world.resource(counter()), 1);
}

// TODO: We'd need something like should_update for this to work, where we compare Child with Child
// #[test]
// fn update_state_on_root_shouldnt_rerender_child() {

//     #[derive(Debug, Clone)]
//     pub struct Root;
//     impl Part for Root {
//         fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
//             let (state, set_state) = hooks.use_state(0);
//             Element::new().listener(trigger(), Arc::new(move |_| {
//                 set_state(state + 1);
//              }))
//                 .children(vec![Child.into()])
//         }
//     }

//     #[derive(Debug, Clone)]
//     pub struct Child;
//     impl Part for Child {
//         fn render(self: Box<Self>, world: &mut World, hooks: &mut Hooks) -> Element {
//             *world.resource_mut(counter()) += 1;
//             Element::new()
//         }
//     }

//     let mut world = initialize();
//     let renderer = Root.el().spawn_tree(&mut world);

//     run_triggers(&mut world);
//     tree.update(&mut world);
//     assert_eq!(*world.resource(counter()), 1);
// }
