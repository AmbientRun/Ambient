use kiwi_element::{Element, ElementComponent, ElementComponentExt, Hooks};
mod common;
use std::sync::Arc;

use common::*;

#[test]
fn basic_context() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            hooks.provide_context(|| 5_u32);
            Child.el()
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child;
    impl ElementComponent for Child {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let _ = hooks.consume_context::<u32>();
            Element::new()
        }
    }

    let mut world = initialize();
    Root.el().spawn_tree(&mut world);
}

#[test]
fn update_context_on_removed_element() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (state, set_state) = hooks.use_state::<u32>(0);
            hooks.provide_context(|| state);
            if state < 3 {
                Element::new().children(vec![Child.into()]).listener(
                    trigger(),
                    Arc::new(move |_| {
                        set_state(state + 1);
                    }),
                )
            } else {
                Element::new()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child;
    impl ElementComponent for Child {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let _ = hooks.consume_context::<u32>();
            Element::new()
        }
    }

    let mut world = initialize();
    let mut tree = Root.el().spawn_tree(&mut world);

    tree.update(&mut world);
    run_triggers(&mut world);
    tree.update(&mut world);
    run_triggers(&mut world);
    tree.update(&mut world);
    run_triggers(&mut world);
    tree.update(&mut world);
    run_triggers(&mut world);
    tree.update(&mut world);
    run_triggers(&mut world);
    tree.update(&mut world);
    run_triggers(&mut world);
}

#[test]
fn two_contexts() {
    #[derive(Debug, Clone)]
    pub struct Root;
    impl ElementComponent for Root {
        fn render(self: Box<Self>, _hooks: &mut Hooks) -> Element {
            Element::new().children(vec![ContextRoot { value: 2 }.el(), ContextRoot { value: 3 }.el()])
        }
    }

    #[derive(Debug, Clone)]
    pub struct ContextRoot {
        value: u32,
    }
    impl ElementComponent for ContextRoot {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            hooks.provide_context(|| self.value);
            Child { value: self.value }.el()
        }
    }

    #[derive(Debug, Clone)]
    pub struct Child {
        value: u32,
    }
    impl ElementComponent for Child {
        fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
            let (ctx_value, _) = hooks.consume_context::<u32>().unwrap();
            assert_eq!(self.value, ctx_value);
            Element::new()
        }
    }

    let mut world = initialize();
    Root.el().spawn_tree(&mut world);
}
