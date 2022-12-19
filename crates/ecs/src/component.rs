// TODO(mithun): I spent two hours trying to make `PartialEq` work correctly
// with a reference to an unboxed IComponent (which is what you get when you remove
// the borrowed boxes.)
// At some point, we should revisit this and try to make it work again, but that was not
// a good use of time.
// error[E0277]: can't compare `&dyn elements_ecs::IComponent` with `elements_ecs::Component<dims_game_objects::player_input::PlayerInput>`
//     = help: the trait `std::cmp::PartialEq<elements_ecs::Component<dims_game_objects::player_input::PlayerInput>>` is not implemented for `&dyn elements_ecs::IComponent`
//     = help: the following other types implement trait `std::cmp::PartialEq<Rhs>`:
//               <(dyn elements_ecs::IComponent + 'a) as std::cmp::PartialEq<elements_ecs::Component<T>>>
//               <(dyn elements_ecs::IComponent + 'a) as std::cmp::PartialEq>
#![allow(clippy::borrowed_box)]

use std::{
    any::Any, collections::HashMap, convert::TryInto, fmt::{self, Display}, marker::PhantomData
};

use downcast_rs::{impl_downcast, Downcast};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use serde::{
    de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor}, Deserializer, Serializer
};

use super::*;

pub trait ComponentValueBase: Send + Sync + Downcast + 'static {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Send + Sync + 'static> ComponentValueBase for T {}
pub trait ComponentValue: ComponentValueBase + Clone {}
impl<T: ComponentValueBase + Clone> ComponentValue for T {}

/// ExComponentValues support serilization, cloning, debug
pub trait ExComponentValue: ComponentValue + Serialize + DeserializeOwned + Clone + std::fmt::Debug {}
impl<T: ComponentValue + Serialize + DeserializeOwned + Clone + std::fmt::Debug> ExComponentValue for T {}

impl_downcast!(IComponent);
impl_downcast!(ComponentValueBase);

pub trait IComponent: Send + Sync + Downcast {
    fn create_buffer(&self) -> Box<dyn IComponentBuffer>;
    fn get_index(&self) -> usize;
    // required for dynamic registration. do not call on static components
    fn set_index(&mut self, index: usize);
    fn get_name(&self) -> String;
    fn is_change_filter(&self) -> bool;
    fn clone_boxed(&self) -> Box<dyn IComponent>;

    fn create_buffer_with_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn IComponentBuffer>;
    fn is_valid_value(&self, value: &Box<dyn ComponentValueBase>) -> bool;
    fn clone_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn ComponentValueBase>;
    fn clone_value_from_world(&self, world: &World, entity: EntityId) -> Result<Box<dyn ComponentValueBase>, ECSError>;
    fn set_at_entity(
        &self,
        world: &mut World,
        entity: EntityId,
        value: &Box<dyn ComponentValueBase>,
    ) -> Result<Box<dyn ComponentValueBase>, ECSError>;
    fn add_component_to_entity(&self, world: &mut World, entity: EntityId, value: &Box<dyn ComponentValueBase>) -> Result<(), ECSError>;
    fn remove_component_from_entity(&self, world: &mut World, entity: EntityId) -> Result<(), ECSError>;

    fn serialize_value<'a>(&self, value: &'a dyn ComponentValueBase) -> &'a dyn erased_serde::Serialize;
    fn deserialize_seq_value(
        &self,
        seq: &mut dyn erased_serde::de::SeqAccess,
    ) -> Result<Option<Box<dyn ComponentValueBase>>, erased_serde::Error>;
    fn deserialize_map_value(&self, seq: &mut dyn erased_serde::de::MapAccess) -> Result<Box<dyn ComponentValueBase>, erased_serde::Error>;
    fn value_to_json_value(&self, value: &Box<dyn ComponentValueBase>) -> serde_json::Value;
    fn value_from_json_value(&self, value: serde_json::Value) -> Result<Box<dyn ComponentValueBase>, serde_json::Error>;
    fn debug_value(&self, value: &Box<dyn ComponentValueBase>) -> String;
    /// I.e. supports serialize, deserialize
    fn is_extended(&self) -> bool;
}

pub struct Component<T: ComponentValue> {
    pub index: i32,
    pub(super) changed_filter: bool,
    name: Option<&'static str>,
    _type: PhantomData<T>,
}

impl<T: ComponentValue> Debug for Component<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Component")
            .field("index", &self.index)
            .field("changed_filter", &self.changed_filter)
            .field("type", &std::any::type_name::<T>())
            .field("name", &self.name.map(|x| x.to_string()).unwrap_or_else(|| component_name(self)))
            .finish()
    }
}
impl<T: ComponentValue> Component<T> {
    pub const fn new(index: i32) -> Self {
        Self { index, changed_filter: false, name: None, _type: PhantomData }
    }
    pub const fn new_with_name(index: i32, name: &'static str) -> Self {
        Self { index, changed_filter: false, name: Some(name), _type: PhantomData }
    }
    pub fn changed(&self) -> Component<T> {
        Self { index: self.index, changed_filter: true, name: self.name.clone(), _type: PhantomData }
    }
    pub fn with(&self, value: T) -> EntityData {
        EntityData::new().set(*self, value)
    }
}
impl<T: ComponentValue + Default> Component<T> {
    pub fn with_default(&self) -> EntityData {
        EntityData::new().set(*self, T::default())
    }
}
impl<T: ComponentValue> Copy for Component<T> {}
impl<T: ComponentValue> Clone for Component<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: ComponentValue> std::hash::Hash for Component<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.changed_filter.hash(state);
        self._type.hash(state);
    }
}
impl<T: ComponentValue> PartialEq for Component<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_index() == other.get_index()
    }
}
impl<T: ComponentValue> PartialEq<Box<dyn IComponent>> for Component<T> {
    fn eq(&self, other: &Box<dyn IComponent>) -> bool {
        self.get_index() == other.get_index()
    }
}
impl<T: ComponentValue> PartialEq<Component<T>> for Box<dyn IComponent> {
    fn eq(&self, other: &Component<T>) -> bool {
        self.get_index() == other.get_index()
    }
}
impl<T: ComponentValue> Eq for Component<T> {}
impl<T: ComponentValue> Ord for Component<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_index().cmp(&other.get_index())
    }
}
impl<T: ComponentValue> PartialOrd for Component<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: ComponentValue> Serialize for Component<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&with_component_registry(|r| r.get_id_for(self).to_owned()))
    }
}
impl<'de, T: ComponentValue> Deserialize<'de> for Component<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ComponentVisitor<T: ComponentValue>(PhantomData<T>);

        impl<'de, T: ComponentValue> Visitor<'de> for ComponentVisitor<T> {
            type Value = Component<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Component<T>")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let component = with_component_registry_mut(|r| Some(r.get_by_id(v)?.clone_boxed()));
                let component = match component {
                    Some(comp) => comp,
                    None => panic!("No such component: {}", v),
                };
                Ok(Component::<T> { index: component.get_index() as i32, name: None, _type: PhantomData, changed_filter: false })
            }
        }

        deserializer.deserialize_str(ComponentVisitor::<T>(PhantomData))
    }
}
impl<T: ComponentValue> IComponent for Component<T> {
    fn create_buffer(&self) -> Box<dyn IComponentBuffer> {
        Box::new(ComponentBuffer::<T>::new(*self))
    }
    fn get_index(&self) -> usize {
        #[cfg(debug_assertions)]
        if self.index < 0 {
            panic!("Component not initialized: {:?}", self.name);
        }
        self.index as usize
    }
    fn set_index(&mut self, index: usize) {
        self.index = index.try_into().unwrap();
    }
    fn get_name(&self) -> String {
        self.name
            .map(|x| x.to_string())
            .unwrap_or_else(|| with_component_registry(|r| r.idx_to_id().get(&self.get_index()).cloned().unwrap()))
    }
    fn is_change_filter(&self) -> bool {
        self.changed_filter
    }
    fn clone_boxed(&self) -> Box<dyn IComponent> {
        Box::new(*self)
    }
    fn create_buffer_with_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn IComponentBuffer> {
        let value = value.downcast_ref::<T>().unwrap();
        Box::new(ComponentBuffer::new_with_value(*self, value.clone()))
    }
    fn is_valid_value(&self, value: &Box<dyn ComponentValueBase>) -> bool {
        value.downcast_ref::<T>().is_some()
    }
    fn clone_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn ComponentValueBase> {
        let value = value.downcast_ref::<T>().unwrap();
        Box::new(value.clone())
    }
    fn clone_value_from_world(&self, world: &World, entity: EntityId) -> Result<Box<dyn ComponentValueBase>, ECSError> {
        world.get_ref(entity, *self).map(|x| Box::new(x.clone()) as Box<dyn ComponentValueBase>)
    }
    fn set_at_entity(
        &self,
        world: &mut World,
        entity: EntityId,
        value: &Box<dyn ComponentValueBase>,
    ) -> Result<Box<dyn ComponentValueBase>, ECSError> {
        let value = value.downcast_ref::<T>().unwrap();
        Ok(Box::new(world.set(entity, *self, value.clone())?))
    }
    fn add_component_to_entity(&self, world: &mut World, entity: EntityId, value: &Box<dyn ComponentValueBase>) -> Result<(), ECSError> {
        let value = value.downcast_ref::<T>().unwrap();
        world.add_component(entity, *self, value.clone())
    }
    fn remove_component_from_entity(&self, world: &mut World, entity: EntityId) -> Result<(), ECSError> {
        world.remove_component(entity, *self)
    }
    default fn serialize_value<'a>(&self, _value: &'a dyn ComponentValueBase) -> &'a dyn erased_serde::Serialize {
        panic!("Component '{}' is not an extended component", self.get_index())
    }
    default fn deserialize_seq_value(
        &self,
        _: &mut dyn erased_serde::de::SeqAccess,
    ) -> Result<Option<Box<dyn ComponentValueBase>>, erased_serde::Error> {
        panic!("Component '{}' is not an extended component", self.get_index())
    }
    default fn deserialize_map_value(
        &self,
        _: &mut dyn erased_serde::de::MapAccess,
    ) -> Result<Box<dyn ComponentValueBase>, erased_serde::Error> {
        panic!("Component '{}' is not an extended component", self.get_index())
    }
    default fn debug_value(&self, _value: &Box<dyn ComponentValueBase>) -> String {
        panic!("Component '{}' is not an extended component", self.get_index())
    }
    default fn is_extended(&self) -> bool {
        false
    }
    default fn value_to_json_value(&self, _value: &Box<dyn ComponentValueBase>) -> serde_json::Value {
        panic!("Component '{}' is not an extended component", self.get_index())
    }
    default fn value_from_json_value(&self, _value: serde_json::Value) -> Result<Box<dyn ComponentValueBase>, serde_json::Error> {
        panic!("Component '{}' is not an extended component", self.get_index())
    }
}
impl<T: ExComponentValue> IComponent for Component<T> {
    fn serialize_value<'a>(&self, value: &'a dyn ComponentValueBase) -> &'a dyn erased_serde::Serialize {
        value.downcast_ref::<T>().expect("Failed to downcast to concrete type")
    }
    fn deserialize_seq_value(
        &self,
        mut seq: &mut dyn erased_serde::de::SeqAccess,
    ) -> Result<Option<Box<dyn ComponentValueBase>>, erased_serde::Error> {
        match seq.next_element::<T>() {
            Ok(Some(value)) => Ok(Some(Box::new(value))),
            Ok(None) => Ok(None),
            Err(err) => Err(erased_serde::de::erase(err)),
        }
    }
    fn deserialize_map_value(
        &self,
        mut map: &mut dyn erased_serde::de::MapAccess,
    ) -> Result<Box<dyn ComponentValueBase>, erased_serde::Error> {
        match map.next_value::<T>() {
            Ok(value) => Ok(Box::new(value)),
            Err(err) => Err(erased_serde::de::erase(err)),
        }
    }
    fn debug_value(&self, value: &Box<dyn ComponentValueBase>) -> String {
        let value = value.downcast_ref::<T>().unwrap();
        format!("{:?}", value)
    }
    fn is_extended(&self) -> bool {
        true
    }
    default fn value_to_json_value(&self, value: &Box<dyn ComponentValueBase>) -> serde_json::Value {
        let value = value.downcast_ref::<T>().unwrap();
        serde_json::to_value(value).unwrap()
    }
    default fn value_from_json_value(&self, value: serde_json::Value) -> Result<Box<dyn ComponentValueBase>, serde_json::Error> {
        let value: T = serde_json::from_value(value)?;
        Ok(Box::new(value))
    }
}

impl PartialEq for dyn IComponent {
    fn eq(&self, other: &Self) -> bool {
        self.get_index() == other.get_index()
    }
}
// From: https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186
impl PartialEq<&Self> for Box<dyn IComponent> {
    fn eq(&self, other: &&Self) -> bool {
        self.get_index() == other.get_index()
    }
}
impl Eq for dyn IComponent {}
impl std::hash::Hash for dyn IComponent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_index().hash(state)
    }
}
impl Clone for Box<dyn IComponent> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}
impl std::fmt::Debug for Box<dyn IComponent> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Box<dyn IComponent>").field("index", &self.get_index()).finish()
    }
}
impl std::fmt::Debug for &dyn IComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("&dyn IComponent").field("index", &self.get_index()).finish()
    }
}
impl Serialize for Box<dyn IComponent> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&with_component_registry(|r| r.get_id_for(self.as_ref()).to_owned()))
    }
}
impl<'de> Deserialize<'de> for Box<dyn IComponent> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BoxIComponentVisitor;

        impl<'de> Visitor<'de> for BoxIComponentVisitor {
            type Value = Box<dyn IComponent>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Box<dyn IComponent>")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let component = with_component_registry_mut(|r| Some(r.get_by_id(v)?.clone_boxed()));
                match component {
                    Some(comp) => Ok(comp),
                    None => Err(de::Error::custom(format!("No such component: {}", v))),
                }
            }
        }

        deserializer.deserialize_str(BoxIComponentVisitor)
    }
}
impl<C: ComponentValue> From<Component<C>> for Box<dyn IComponent> {
    fn from(comp: Component<C>) -> Self {
        comp.clone_boxed()
    }
}

pub trait IComponentBuffer: Send + Sync {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn component_index(&self) -> usize;
    fn component_boxed(&self) -> Box<dyn IComponent>;
    fn append(&mut self, buffer: Box<dyn IComponentBuffer>, count: usize);
    fn set(&mut self, index: usize, value: &Box<dyn ComponentValueBase>);
    fn swap_remove_index(&mut self, index: usize) -> Box<dyn IComponentBuffer>;
    fn remove_index_boxed(&mut self, index: usize) -> Box<dyn ComponentValueBase>;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
    fn write_to_world(self: Box<Self>, world: &mut World, entity: EntityId) -> Result<(), ECSError>;
    fn clone_boxed(&self) -> Box<dyn IComponentBuffer>;
    fn clone_value_boxed(&self, index: usize) -> Box<dyn ComponentValueBase>;
    fn pop_unit(&mut self) -> ComponentUnit;
    fn dump_index(&self, index: usize) -> String;
}

#[derive(Debug, Clone)]
pub struct ComponentBuffer<T: ComponentValue> {
    pub component: Component<T>,
    pub data: Vec<T>,
}
impl<T: ComponentValue> ComponentBuffer<T> {
    pub fn new(component: Component<T>) -> Self {
        Self { component, data: Vec::new() }
    }
    pub fn new_with_value(component: Component<T>, value: T) -> Self {
        Self { component, data: vec![value] }
    }
}
impl<T: ComponentValue> IComponentBuffer for ComponentBuffer<T> {
    fn len(&self) -> usize {
        self.data.len()
    }
    fn component_index(&self) -> usize {
        self.component.get_index()
    }
    fn component_boxed(&self) -> Box<dyn IComponent> {
        Box::new(self.component)
    }
    fn append(&mut self, mut buffer: Box<dyn IComponentBuffer>, count: usize) {
        let b = buffer.as_mut_any().downcast_mut::<ComponentBuffer<T>>().unwrap();
        let x = b.data.pop().unwrap();
        self.data.resize(self.data.len() + count, x);
    }
    fn set(&mut self, index: usize, value: &Box<dyn ComponentValueBase>) {
        let b = value.downcast_ref::<T>().unwrap();
        self.data[index] = b.clone();
    }
    fn swap_remove_index(&mut self, index: usize) -> Box<dyn IComponentBuffer> {
        let value = self.data.swap_remove(index);
        Box::new(ComponentBuffer::new_with_value(self.component, value))
    }
    fn remove_index_boxed(&mut self, index: usize) -> Box<dyn ComponentValueBase> {
        Box::new(self.data.remove(index))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
    fn write_to_world(mut self: Box<Self>, world: &mut World, entity: EntityId) -> Result<(), ECSError> {
        world.set(entity, self.component, self.data.pop().unwrap())?;
        Ok(())
    }
    fn clone_boxed(&self) -> Box<dyn IComponentBuffer> {
        Box::new(self.clone())
    }
    fn clone_value_boxed(&self, index: usize) -> Box<dyn ComponentValueBase> {
        Box::new(self.data[index].clone())
    }
    fn pop_unit(&mut self) -> ComponentUnit {
        ComponentUnit::new(self.component, self.data.pop().unwrap())
    }

    default fn dump_index(&self, _index: usize) -> String {
        "-".to_string()
    }
}
impl<T: ComponentValue + Debug> IComponentBuffer for ComponentBuffer<T> {
    default fn dump_index(&self, index: usize) -> String {
        format!("{:?}", self.data[index])
    }
}
impl<T: ComponentValue + Debug + Display> IComponentBuffer for ComponentBuffer<T> {
    fn dump_index(&self, index: usize) -> String {
        format!("{}", self.data[index])
    }
}

impl Clone for Box<dyn IComponentBuffer> {
    fn clone(&self) -> Self {
        self.as_ref().clone_boxed()
    }
}

static COMPONENT_REGISTRY: RwLock<OnceCell<Box<dyn IComponentRegistry + Send + Sync>>> = RwLock::new(OnceCell::new());
pub fn with_component_registry<R>(f: impl FnOnce(&dyn IComponentRegistry) -> R + Sync + Send) -> R {
    let lock = COMPONENT_REGISTRY.read();
    f(lock.get().expect("component registry not initialized").as_ref())
}
pub fn with_component_registry_mut<R>(f: impl FnOnce(&mut dyn IComponentRegistry) -> R + Sync + Send) -> R {
    let mut lock = COMPONENT_REGISTRY.write();
    f(lock.get_mut().expect("component registry not initialized").as_mut())
}
pub fn set_component_registry<R: IComponentRegistry + Send + Sync + 'static>(registry: impl FnOnce() -> R) {
    let lock = COMPONENT_REGISTRY.write();
    if lock.get().is_none() && lock.set(Box::new(registry())).is_err() {
        panic!("component registry already set");
    }
}

pub trait IComponentRegistry {
    fn register_with_id(&mut self, id: &str, component: &mut dyn IComponent);
    fn register_internal_with_namespace(&mut self, namespace: &str, name: &str, component: &mut dyn IComponent);

    /// mutable as the registry may need to modify its own state to service this request
    fn get_by_id(&mut self, id: &str) -> Option<&dyn IComponent>;
    /// use [get_by_id] unless you are sure that your component does not need to be loaded in from another location
    fn get_by_id_without_load(&self, id: &str) -> Option<&dyn IComponent>;
    fn get_by_index(&self, index: usize) -> Option<&dyn IComponent>;
    fn all(&self) -> &[Box<dyn IComponent>];
    fn idx_to_id(&self) -> &HashMap<usize, String>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
pub trait IComponentRegistryExt: IComponentRegistry {
    fn register<T: ComponentValue>(&mut self, namespace: &str, name: &str, component: &mut Component<T>) {
        if component.index >= 0 {
            return;
        }
        self.register_internal_with_namespace(namespace, name, &mut *component);
    }
    fn get_by_id_type<T: IComponent + Clone>(&mut self, id: &str) -> Option<T> {
        self.get_by_id(id)?.downcast_ref().cloned()
    }
    fn get_by_index_type<T: IComponent + Clone>(&self, index: usize) -> Option<T> {
        self.get_by_index(index)?.downcast_ref().cloned()
    }
    fn get_id_for_opt(&self, component: &dyn IComponent) -> Option<&str> {
        self.idx_to_id().get(&component.get_index()).map(|s| s.as_str())
    }
    /// Will panic if the specified component does not exist
    fn get_id_for(&self, component: &dyn IComponent) -> &str {
        match self.get_id_for_opt(component) {
            Some(id) => id,
            None => panic!("failed to get id for component {}", component.get_index()),
        }
    }
    fn component_count(&self) -> usize {
        self.all().len()
    }
}
impl<I: ?Sized> IComponentRegistryExt for I where I: IComponentRegistry {}

#[derive(Clone, Default)]
pub struct SimpleComponentRegistry {
    components_by_name: HashMap<String, Box<dyn IComponent>>,
    idx_to_id: HashMap<usize, String>,
    components: Vec<Box<dyn IComponent>>,
}
impl SimpleComponentRegistry {
    pub fn install() {
        set_component_registry(Self::default);
    }
}
impl IComponentRegistry for SimpleComponentRegistry {
    fn register_with_id(&mut self, id: &str, component: &mut dyn IComponent) {
        if self.components_by_name.contains_key(id) {
            log::warn!("Duplicate components: {}", id);
            return;
        }
        component.set_index(self.components.len());
        self.components.push(component.clone_boxed());
        self.components_by_name.insert(id.to_owned(), component.clone_boxed());
        self.idx_to_id.insert(component.get_index(), id.to_owned());
    }
    fn register_internal_with_namespace(&mut self, namespace: &str, name: &str, component: &mut dyn IComponent) {
        self.register_with_id(&format!("{namespace}::{name}"), component)
    }

    fn get_by_id(&mut self, id: &str) -> Option<&dyn IComponent> {
        self.get_by_id_without_load(id)
    }
    fn get_by_id_without_load(&self, id: &str) -> Option<&dyn IComponent> {
        self.components_by_name.get(id).map(|b| b.as_ref())
    }
    fn get_by_index(&self, index: usize) -> Option<&dyn IComponent> {
        self.components.get(index).map(|b| b.as_ref())
    }
    fn all(&self) -> &[Box<dyn IComponent>] {
        &self.components
    }
    fn idx_to_id(&self) -> &HashMap<usize, String> {
        &self.idx_to_id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[macro_export]
macro_rules! components {
    ( $namespace:literal, { $( $(#[$outer:meta])* $name:ident : $ty:ty, )+ } ) => {
        $(
            $crate::paste::paste! {
                #[allow(non_upper_case_globals)]
                #[no_mangle]
                static mut [<comp_ $name>]: $crate::Component<$ty> = $crate::Component::new_with_name(-1, stringify!($name));
            }
            $(#[$outer])*
            pub fn $name() -> $crate::Component<$ty> {
                $crate::paste::paste! {
                    unsafe { [<comp_ $name>] }
                }
            }
        )*
        /// Initialize the components for the module
        static COMPONENTS_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        pub fn init_components() {
            use std::sync::atomic::Ordering;
            use $crate::IComponentRegistryExt;

            if COMPONENTS_INITIALIZED.load(Ordering::SeqCst) {
                return;
            }

            $crate::with_component_registry_mut(|registry| unsafe {
                $(
                    $crate::paste::paste! {
                        registry.register(concat!("core::", $namespace), stringify!($name), &mut [<comp_ $name>]);
                    }
                )*
            });
            COMPONENTS_INITIALIZED.store(true, Ordering::SeqCst);
        }
    };
}
