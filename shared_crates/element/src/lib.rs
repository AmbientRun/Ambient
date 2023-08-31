//! Element is a React-inspired virtual tree library for the Ambient runtime.
//!
//! It is backed by the Ambient ECS; the virtual tree is converted into a real tree of entities and components.
//! When the tree is updated, it is compared to the previous tree, and only the differences are applied to the ECS.
//! This can be used for UI, as well as any other tree-like data structure that you want to be able to update efficiently.
//!
//! # Idioms
//!
//! By convention, most [ElementComponent]s define an `el` method that returns an [Element] of that type. This `el`
//! takes the properties to make it easy to both construct the component and instantiate it as an [Element].
//!
//! In addition to this, [ElementComponentExt] adds an `el` method to all [ElementComponent]s that converts them to
//! an [Element].
//!
//! This means that an [ElementComponent] that looks like this
//! ```ignore
//! #[element_component]
//! fn MyComponent(hooks: &mut Hooks, a: u32, b: String) -> Element {
//!    // ...
//! }
//! ```
//!
//! can be instantiated as an [Element] using either of these methods:
//! ```ignore
//! MyComponent { a: 42, b: "hello".to_string() }.el()
//! ```
//! or
//! ```ignore
//! MyComponent::el(42, "hello".to_string())
//! ```
//!
//! # Passing data in
//!
//! To pass data into the root of an Element tree, pass the data into its properties when constructing it and/or update the root
//! of the tree using [ElementTree::migrate_root].
//!
//! To receive data from an Element tree, we recommend you use messaging. This includes sending messages to the server and/or
//! standard messaging channels in Rust (e.g. `std::sync::mpsc::channel`). We do not generally recommend trying to send data
//! out of the tree directly, as this can be difficult to reason about.

#![deny(missing_docs)]

#[macro_use]
extern crate derivative;

use std::{any::Any, sync::Arc};

#[cfg(feature = "native")]
use ambient_guest_bridge::ecs::{components, SystemGroup};
use ambient_guest_bridge::ecs::{
    Component, ComponentDesc, ComponentValue, Entity, EntityId, World,
};
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
    /// The element tree state for an entity.
    element_tree: ShareableElementTree,
});
pub use ambient_guest_bridge::core::app::components::{element, element_unmanaged_children};

#[clonable]
/// A trait for types that can be converted to `Any` and can also be cloned.
pub trait AnyCloneable: AsAny + Clone + std::fmt::Debug {}
impl<T: Clone + std::fmt::Debug + Any + 'static> AnyCloneable for T {}
impl as_any::Downcast for dyn AnyCloneable {}
impl as_any::Downcast for dyn AnyCloneable + Send {}
impl as_any::Downcast for dyn AnyCloneable + Sync {}
impl as_any::Downcast for dyn AnyCloneable + Send + Sync {}

type InstanceId = String;

#[clonable]
/// The base trait for all element components. These are similar to React components.
///
/// The `render` method is called to create the virtual tree for this component.
/// It will only be called when the component is first created, or when one of its dependencies changes.
/// These dependencies can include properties or state introduced by [Hooks].
pub trait ElementComponent: std::fmt::Debug + ElementComponentName + Clone + Sync + Send {
    /// Render the virtual tree for this component.
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element;
}
/// Contains the name of the type implementing [ElementComponent].
pub trait ElementComponentName {
    /// Returns the name of the type implementing [ElementComponent].
    fn element_component_name(&self) -> &'static str;
}
impl<T> ElementComponentName for T {
    fn element_component_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
impl<T: ElementComponent + 'static> From<T> for Element {
    fn from(part: T) -> Self {
        Element::from_element_component(Box::new(part))
    }
}

/// A convenience trait for converting an [ElementComponent] into an [Element].
///
/// For more information on this, see the [top-level documentation](crate).
pub trait ElementComponentExt {
    /// Converts an [ElementComponent] into an [Element].
    fn el(self) -> Element;
}
impl<T: ElementComponent + 'static> ElementComponentExt for T {
    fn el(self) -> Element {
        Element::from(self)
    }
}

#[derive(Clone, Debug)]
/// A rendered [ElementComponent] instance.
pub struct Element {
    config: ElementConfig,
    children: Vec<Element>,
}
impl Element {
    /// Creates a new [Element] with no children.
    pub fn new() -> Self {
        Self {
            config: ElementConfig::new(),
            children: Vec::new(),
        }
    }
    /// Creates a new [Element] from the given component.
    pub fn from_element_component(part: Box<dyn ElementComponent>) -> Self {
        let mut s = Self::new();
        s.config.part = Some(part);
        s
    }
    /// Convenience method to construct a `Vec<Element>` from a single [Element].
    pub fn vec_of(self) -> Vec<Self> {
        vec![self]
    }
    /// Adds the given `component` with `value` to the element.
    pub fn with<T: ComponentValue + Sync + Send + Clone + 'static>(
        mut self,
        component: Component<T>,
        value: T,
    ) -> Self {
        self.config.components.set(component, value);
        self
    }
    /// Sets the given `component` to `value` on the element during initialization only.
    pub fn init<T: ComponentValue + Sync + Send + Clone + 'static>(
        mut self,
        component: Component<T>,
        value: T,
    ) -> Self {
        self.config.init_components.set(component, value);
        self
    }
    /// Calls [Self::init] with the default value for the component's type.
    pub fn init_default<T: ComponentValue + Sync + Send + Clone + Default + 'static>(
        self,
        component: Component<T>,
    ) -> Self {
        self.init(component, T::default())
    }
    #[cfg(feature = "native")]
    /// Extends the element with all of the values from the given [Entity].
    pub fn extend(mut self, entity_data: Entity) -> Self {
        for unit in entity_data.into_iter() {
            self.config.components.set_writer(
                unit.desc(),
                Arc::new(move |_, ed| ed.set_entry(unit.clone())),
            );
        }
        self
    }
    /// See [`Element::init`]; adds each entry in the Entity to init
    #[cfg(feature = "native")]
    pub fn init_extend(mut self, entity_data: Entity) -> Self {
        for unit in entity_data.into_iter() {
            self.config.init_components.set_writer(
                unit.desc(),
                Arc::new(move |_, ed| ed.set_entry(unit.clone())),
            );
        }
        self
    }

    /// Removes the given `component` from the element.
    ///
    /// Warning: this only removes components on the current element.
    // TODO: Make this remove components on the super element too.
    pub fn remove<T: ComponentValue + Clone>(mut self, component: Component<T>) -> Self {
        self.config.components.remove(component);
        self.config.init_components.remove(component);
        self
    }
    /// Sets the children of the element.
    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }
    /// Set the function used to spawn the element.
    pub fn spawner<F: Fn(&mut World, Entity) -> EntityId + Sync + Send + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.config.spawner = Arc::new(handler);
        self
    }
    /// Set the function used to despawn the element.
    pub fn despawner<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.config.despawner = Arc::new(handler);
        self
    }
    /// Set the callback to call when the element is spawned.
    pub fn on_spawned<F: Fn(&mut World, EntityId, &str) + Sync + Send + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.config.on_spawned = Some(Arc::new(handler));
        self
    }
    /// Set the callback to call when the element is despawned.
    pub fn on_despawn<F: Fn(&mut World, EntityId, &str) + Sync + Send + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.config.on_despawn = Some(Arc::new(handler));
        self
    }
    /// Set the unique key used to identify this element.
    ///
    /// This is used to disambiguate elements with the same type. This should be used when rendering lists of elements.
    pub fn key<T: Into<String>>(mut self, key: T) -> Self {
        self.config.key = key.into();
        self
    }
    /// Avoid rendering the subtree, except when the memo_key is changed.
    pub fn memoize_subtree(mut self, memo_key: impl Into<String>) -> Self {
        self.config.memo_key = Some(memo_key.into());
        self
    }
    /// Returns true if the element has the given `component`.
    pub fn has_component(&self, component: impl Into<ComponentDesc>) -> bool {
        let index = component.into().index() as usize;
        self.config.components.0.contains_key(&index)
            || self.config.init_components.0.contains_key(&index)
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
        let entity = Entity::new().with(
            self::element_tree(),
            ShareableElementTree(Arc::new(Mutex::new(tree))),
        );
        world.spawn(entity)
    }
    /// This spawns the element tree and returns it. The tree won't be automatically updated, but can manually be updated
    /// by calling the `update` method.
    pub fn spawn_tree(self, world: &mut World) -> ElementTree {
        ElementTree::new(world, self)
    }
    /// This spawns the element tree and sets up listeners to automatically update it.
    ///
    /// This is equivalent to calling [Self::spawn_tree] and then calling [ElementTree::update] on the tree each frame.
    ///
    /// You may want to update the tree manually if you want to replace the root [Element]:
    /// ```ignore
    /// let mut tree = Element::new().spawn_tree();
    /// Frame::subscribe(move |_| {
    ///     if some_condition {
    ///         tree.migrate_root(&mut World, App::el(new_properties));
    ///     }
    ///     tree.update(&mut World);
    /// });
    /// ```
    #[cfg(feature = "guest")]
    pub fn spawn_interactive(self, world: &mut World) {
        use ambient_guest_bridge::api::{
            core::messages::Frame, message::RuntimeMessage, prelude::OkEmpty,
        };

        let mut tree = self.spawn_tree(world);
        Frame::subscribe(move |_| {
            tree.update(world);
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
/// The systems required to drive [ElementTree]s.
pub fn ambient_system() -> SystemGroup {
    ElementTree::systems_for_component(element_tree())
}

#[macro_export]
/// Helper macro to define a `el` function for a newtype of a vector of [Element]s.
#[doc(hidden)]
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
/// Render the given tree underneath `id`.
pub fn render_parented_with_component(
    world: &mut World,
    id: EntityId,
    handle: Component<ShareableElementTree>,
    mut element: Element,
) {
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
    if let Ok(tree) = world.get_cloned(id, handle) {
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
        world
            .add_component_if_required(id, local_to_world(), Default::default())
            .unwrap();
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
