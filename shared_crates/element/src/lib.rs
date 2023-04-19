#[macro_use]
extern crate derivative;

use std::{any::Any, sync::Arc};

#[cfg(feature = "native")]
use ambient_guest_bridge::ecs::{components, SystemGroup};
use ambient_guest_bridge::ecs::{Component, ComponentDesc, ComponentValue, Entity, EntityId, World};
use as_any::AsAny;
use dyn_clonable::clonable;
#[cfg(feature = "native")]
use parking_lot::Mutex;

mod element_config;
mod hooks;
mod standard;
mod tree;
pub use ambient_element_component::element_component;
use element_config::*;
pub use hooks::*;
pub use standard::*;
pub use tree::*;

#[cfg(feature = "native")]
components!("app", {
    element_tree: ShareableElementTree,
});
pub use ambient_guest_bridge::components::app::{element, element_unmanaged_children};

#[clonable]
pub trait AnyCloneable: AsAny + Clone + std::fmt::Debug {}
impl<T: Clone + std::fmt::Debug + Any + 'static> AnyCloneable for T {}
impl as_any::Downcast for dyn AnyCloneable {}
impl as_any::Downcast for dyn AnyCloneable + Send {}
impl as_any::Downcast for dyn AnyCloneable + Sync {}
impl as_any::Downcast for dyn AnyCloneable + Send + Sync {}

type InstanceId = String;

#[clonable]
pub trait ElementComponent: std::fmt::Debug + PartName + Clone + Sync + Send {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element;
}
pub trait PartName {
    fn part_name(&self) -> &'static str;
}
impl<T> PartName for T {
    fn part_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
impl<T: ElementComponent + 'static> From<T> for Element {
    fn from(part: T) -> Self {
        Element::from_part(Box::new(part))
    }
}
pub trait ElementComponentExt {
    fn el(self) -> Element;
}
impl<T: ElementComponent + 'static> ElementComponentExt for T {
    fn el(self) -> Element {
        Element::from(self)
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    config: ElementConfig,
    children: Vec<Element>,
}
impl Element {
    pub fn new() -> Self {
        Self { config: ElementConfig::new(), children: Vec::new() }
    }
    pub fn from_part(part: Box<dyn ElementComponent>) -> Self {
        let mut s = Self::new();
        s.config.part = Some(part);
        s
    }
    /// convenience method to construct a vec from a single Element
    pub fn vec_of(self) -> Vec<Self> {
        vec![self]
    }
    pub fn with<T: ComponentValue + Sync + Send + Clone + 'static>(mut self, component: Component<T>, value: T) -> Self {
        self.config.components.set(component, value);
        self
    }
    pub fn with_default<T: ComponentValue + Sync + Send + Clone + Default + 'static>(mut self, component: Component<T>) -> Self {
        self.config.components.set(component, T::default());
        self
    }
    /// Sets the component of the element component instantiation
    pub fn init<T: ComponentValue + Sync + Send + Clone + 'static>(mut self, component: Component<T>, value: T) -> Self {
        self.config.init_components.set(component, value);
        self
    }
    /// See [`Element::init`]
    pub fn init_default<T: ComponentValue + Sync + Send + Clone + Default + 'static>(mut self, component: Component<T>) -> Self {
        self.config.init_components.set(component, T::default());
        self
    }
    #[cfg(feature = "native")]
    pub fn extend(mut self, entity_data: Entity) -> Self {
        for unit in entity_data.into_iter() {
            self.config.components.set_writer(unit.desc(), Arc::new(move |_, ed| ed.set_entry(unit.clone())));
        }
        self
    }
    /// See [`Element::init`]; adds each entry in the Entity to init
    #[cfg(feature = "native")]
    pub fn init_extend(mut self, entity_data: Entity) -> Self {
        for unit in entity_data.into_iter() {
            self.config.init_components.set_writer(unit.desc(), Arc::new(move |_, ed| ed.set_entry(unit.clone())));
        }
        self
    }

    /// Warning: this only removes components on the current element.
    ///
    /// TODO: Make this remove components on the super element too.
    pub fn remove<T: ComponentValue + Clone>(mut self, component: Component<T>) -> Self {
        self.config.components.remove(component);
        self.config.init_components.remove(component);
        self
    }
    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }
    pub fn spawner<F: Fn(&mut World, Entity) -> EntityId + Sync + Send + 'static>(mut self, handler: F) -> Self {
        self.config.spawner = Arc::new(handler);
        self
    }
    pub fn despawner<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handler: F) -> Self {
        self.config.despawner = Arc::new(handler);
        self
    }
    pub fn on_spawned<F: Fn(&mut World, EntityId, &str) + Sync + Send + 'static>(mut self, handler: F) -> Self {
        self.config.on_spawned = Some(Arc::new(handler));
        self
    }
    pub fn on_despawn<F: Fn(&mut World, EntityId, &str) + Sync + Send + 'static>(mut self, handler: F) -> Self {
        self.config.on_despawn = Some(Arc::new(handler));
        self
    }
    pub fn key<T: Into<String>>(mut self, key: T) -> Self {
        self.config.key = key.into();
        self
    }
    /// Avoid rendering the subtree, except when the memo_key is changed.
    pub fn memoize_subtree(mut self, memo_key: impl Into<String>) -> Self {
        self.config.memo_key = Some(memo_key.into());
        self
    }
    pub fn has_component(&self, component: impl Into<ComponentDesc>) -> bool {
        let index = component.into().index() as usize;
        self.config.components.0.contains_key(&index) || self.config.init_components.0.contains_key(&index)
    }
    /// This spawns the element tree as a number of entities, but they won't react to changes. Returns the root entity
    #[cfg(feature = "native")]
    pub fn spawn_static(self, world: &mut World) -> EntityId {
        ElementTree::new(world, self).root_entity().unwrap()
    }
    /// This spawns the element tree plus a handle entity which will have an `element_tree` component on it. All
    /// `element_tree` components get updated each frame so this entity tree will be updated
    #[cfg(feature = "native")]
    pub fn spawn_interactive(self, world: &mut World) -> EntityId {
        let tree = self.spawn_tree(world);
        let entity = Entity::new().with(self::element_tree(), ShareableElementTree(Arc::new(Mutex::new(tree))));
        world.spawn(entity)
    }
    /// This spawns the elemet tree and returns it. The tree won't be automatically updated, but can manually be updated
    /// by calling the `update` method.
    #[cfg(feature = "native")]
    pub fn spawn_tree(self, world: &mut World) -> ElementTree {
        ElementTree::new(world, self)
    }
    /// This spawns the elemet tree and returns it. The tree won't be automatically updated, but can manually be updated
    /// by calling the `update` method.
    #[cfg(feature = "guest")]
    pub fn spawn_tree(self) -> ElementTree {
        ElementTree::new(&mut World, self)
    }
    /// This spawns the element tree and sets up listeners to automatically update it.
    #[cfg(feature = "guest")]
    pub fn spawn_interactive(self) {
        use ambient_guest_bridge::api::{message::RuntimeMessage, messages, prelude::OkEmpty};

        let mut tree = self.spawn_tree();
        messages::Frame::subscribe(move |_| {
            tree.update(&mut World);
            OkEmpty
        });
    }
}

impl Default for Element {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "native")]
pub fn ambient_system() -> SystemGroup {
    ElementTree::systems_for_component(element_tree())
}

#[macro_export]
macro_rules! define_el_function_for_vec_element_newtype {
    ($type:ty) => {
        impl $type {
            /// Creates an [Element] of this type from a vector of [Element]s
            pub fn el(contents: impl std::iter::IntoIterator<Item = Element>) -> Element {
                Self(contents.into_iter().collect()).el()
            }
        }
    };
}

#[cfg(feature = "native")]
pub fn render_parented_with_component(world: &mut World, id: EntityId, handle: Component<ShareableElementTree>, mut element: Element) {
    use ambient_core::{
        hierarchy::{children, parent},
        transform::{local_to_parent, local_to_world},
    };
    element = element.with(parent(), id);
    if !element.has_component(local_to_parent()) {
        element = element.init_default(local_to_parent());
    }
    if !element.has_component(local_to_world()) {
        element = element.init_default(local_to_world());
    }
    if let Ok(tree) = world.get_ref(id, handle).map(|x| x.clone()) {
        let mut tree = tree.0.lock();
        let prev_root = tree.root_entity();
        tree.migrate_root(world, element);
        let next_root = tree.root_entity();
        if next_root != prev_root {
            let children = world.get_mut(id, children()).unwrap();
            if let Some(prev_root) = prev_root {
                children.retain(|x| *x != prev_root);
            }
            if let Some(next_root) = next_root {
                children.push(next_root);
            }
        }
    } else {
        let tree = ShareableElementTree::new(world, element);
        world.add_component(id, handle, tree.clone()).unwrap();
        let root = tree.0.lock().root_entity();
        if let Some(root) = root {
            if let Ok(children) = world.get_mut(id, children()) {
                children.push(root);
            } else {
                world.add_component(id, children(), vec![root]).unwrap();
            }
        }
        if !world.has_component(id, local_to_world()) {
            world.add_component(id, local_to_world(), Default::default()).unwrap();
        }
    }
}

#[macro_export]
/// Shorthand for `let x = x.to_owned();`
macro_rules! to_owned {
    ($($es:ident),+) => {$(
        #[allow(unused_mut)]
        let mut $es = $es.to_owned();
    )*}
}
