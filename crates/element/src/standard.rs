use std::collections::HashMap;

use elements_ecs::World;
use itertools::Itertools;

use super::{define_el_function_for_vec_element_newtype, Element, ElementComponent, ElementComponentExt, Hooks};

/// Useful for introducing an intermediate component node in a tree
#[derive(Debug, Clone)]
pub struct Wrap(pub Element);
impl ElementComponent for Wrap {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        self.0
    }
}

#[derive(Debug, Clone)]
/// Wrap multiple elements in a flat hierarchy
pub struct Group(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(Group);
impl ElementComponent for Group {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        Element::new().children(self.0)
    }
}
impl<T: ElementComponent + Clone + 'static> ElementComponent for Vec<T> {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        Group(self.into_iter().map(|part| Element::from(part)).collect_vec()).into()
    }
}
impl<T: ElementComponent + Clone + 'static> ElementComponent for HashMap<String, T> {
    fn render(self: Box<Self>, _: &mut World, _: &mut Hooks) -> Element {
        Group(self.into_iter().sorted_by_key(|x| x.0.clone()).map(|(key, part)| Element::from(part).key(&key)).collect_vec()).into()
    }
}

#[derive(Debug, Clone)]
pub struct Memo<P: ElementComponent + PartialEq + Clone + 'static>(pub P);
impl<P: ElementComponent + PartialEq + Clone + 'static> ElementComponent for Memo<P> {
    fn render(self: Box<Self>, _world: &mut World, _hooks: &mut Hooks) -> Element {
        let key = format!("{:?}", self.0);
        Element::from(self.0).memoize_subtree(key)
    }
}
