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

use std::{self, any::Any};

use downcast_rs::impl_downcast;
use serde::{de::DeserializeOwned, Deserializer, Serializer};

use super::*;
use crate::ComponentEntry;

/// ExComponentValues support serilization, cloning, debug
pub trait ExComponentValue: ComponentValue + Serialize + DeserializeOwned + Clone + std::fmt::Debug {}
impl<T: ComponentValue + Serialize + DeserializeOwned + Clone + std::fmt::Debug> ExComponentValue for T {}

impl_downcast!(ComponentValueBase);

// pub trait IComponent: Send + Sync + Downcast {
//     fn create_buffer(&self) -> Box<dyn IComponentBuffer>;
//     fn get_index(&self) -> usize;
//     fn external_type(&self) -> Option<PrimitiveComponentType>;
//     // required for dynamic registration. do not call on static components
//     fn set_index(&mut self, index: usize);
//     fn get_id(&self) -> String;
//     fn get_name(&self) -> String;
//     fn is_change_filter(&self) -> bool;
//     fn clone_boxed(&self) -> Box<dyn IComponent>;

//     fn create_buffer_with_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn IComponentBuffer>;
//     fn is_valid_value(&self, value: &Box<dyn ComponentValueBase>) -> bool;
//     fn clone_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn ComponentValueBase>;
//     fn clone_value_from_world(&self, world: &World, entity: EntityId) -> Result<Box<dyn ComponentValueBase>, ECSError>;
//     fn set_at_entity(
//         &self,
//         world: &mut World,
//         entity: EntityId,
//         value: &Box<dyn ComponentValueBase>,
//     ) -> Result<Box<dyn ComponentValueBase>, ECSError>;
//     fn add_component_to_entity(&self, world: &mut World, entity: EntityId, value: &Box<dyn ComponentValueBase>) -> Result<(), ECSError>;
//     fn remove_component_from_entity(&self, world: &mut World, entity: EntityId) -> Result<(), ECSError>;

//     fn serialize_value<'a>(&self, value: &'a dyn ComponentValueBase) -> &'a dyn erased_serde::Serialize;
//     fn deserialize_seq_value(
//         &self,
//         seq: &mut dyn erased_serde::de::SeqAccess,
//     ) -> Result<Option<Box<dyn ComponentValueBase>>, erased_serde::Error>;
//     fn deserialize_map_value(&self, seq: &mut dyn erased_serde::de::MapAccess) -> Result<Box<dyn ComponentValueBase>, erased_serde::Error>;
//     fn value_to_json_value(&self, value: &Box<dyn ComponentValueBase>) -> serde_json::Value;
//     fn value_from_json_value(&self, value: serde_json::Value) -> Result<Box<dyn ComponentValueBase>, serde_json::Error>;
//     fn debug_value(&self, value: &Box<dyn ComponentValueBase>) -> String;
//     /// I.e. supports serialize, deserialize
//     fn is_extended(&self) -> bool;
// }

impl<T: ComponentValue + Default> Component<T> {
    pub fn with_default(&self) -> EntityData {
        EntityData::new().set(*self, T::default())
    }
}

impl<T: ComponentValue> Serialize for Component<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.desc().serialize(serializer)
    }
}

impl<'de, T: ComponentValue> Deserialize<'de> for Component<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let desc: ComponentDesc = ComponentDesc::deserialize(deserializer)?;
        Ok(Self::new(desc))
    }
}
// impl<T: ComponentValue> IComponent for Component<T> {
//     fn create_buffer(&self) -> Box<dyn IComponentBuffer> {
//         Box::new(ComponentBuffer::<T>::new(*self))
//     }
//     fn get_index(&self) -> usize {
//         #[cfg(debug_assertions)]
//         if self.index < 0 {
//             panic!("Component not initialized: {:?}", self.name);
//         }
//         self.index as usize
//     }
//     fn external_type(&self) -> Option<PrimitiveComponentType> {
//         todo!()
//         // ComponentRegistry::get().components[self.get_index()].primitive_component_type.clone()
//     }
//     fn set_index(&mut self, index: usize) {
//         self.index = index.try_into().unwrap();
//     }
//     fn get_id(&self) -> String {
//         with_component_registry(|r| r.idx_to_id().get(&self.get_index()).cloned().unwrap())
//     }
//     fn get_name(&self) -> String {
//         self.name
//             .map(|x| x.to_string())
//             .unwrap_or_else(|| with_component_registry(|r| r.idx_to_id().get(&self.get_index()).cloned().unwrap()))
//     }
//     fn is_change_filter(&self) -> bool {
//         self.changed_filter
//     }
//     fn clone_boxed(&self) -> Box<dyn IComponent> {
//         Box::new(*self)
//     }
//     fn create_buffer_with_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn IComponentBuffer> {
//         let value = value.downcast_ref::<T>().unwrap();
//         Box::new(ComponentBuffer::new_with_value(*self, value.clone()))
//     }
//     fn is_valid_value(&self, value: &Box<dyn ComponentValueBase>) -> bool {
//         value.downcast_ref::<T>().is_some()
//     }
//     fn clone_value(&self, value: &Box<dyn ComponentValueBase>) -> Box<dyn ComponentValueBase> {
//         let value = value.downcast_ref::<T>().unwrap();
//         Box::new(value.clone())
//     }
//     fn clone_value_from_world(&self, world: &World, entity: EntityId) -> Result<Box<dyn ComponentValueBase>, ECSError> {
//         world.get_ref(entity, *self).map(|x| Box::new(x.clone()) as Box<dyn ComponentValueBase>)
//     }
//     fn set_at_entity(
//         &self,
//         world: &mut World,
//         entity: EntityId,
//         value: &Box<dyn ComponentValueBase>,
//     ) -> Result<Box<dyn ComponentValueBase>, ECSError> {
//         let value = value.downcast_ref::<T>().unwrap();
//         Ok(Box::new(world.set(entity, *self, value.clone())?))
//     }
//     fn add_component_to_entity(&self, world: &mut World, entity: EntityId, value: &Box<dyn ComponentValueBase>) -> Result<(), ECSError> {
//         let value = value.downcast_ref::<T>().unwrap();
//         world.add_component(entity, *self, value.clone())
//     }
//     fn remove_component_from_entity(&self, world: &mut World, entity: EntityId) -> Result<(), ECSError> {
//         world.remove_component(entity, *self)
//     }
//     default fn serialize_value<'a>(&self, _value: &'a dyn ComponentValueBase) -> &'a dyn erased_serde::Serialize {
//         panic!("Component '{}' is not an extended component", self.get_index())
//     }
//     default fn deserialize_seq_value(
//         &self,
//         _: &mut dyn erased_serde::de::SeqAccess,
//     ) -> Result<Option<Box<dyn ComponentValueBase>>, erased_serde::Error> {
//         panic!("Component '{}' is not an extended component", self.get_index())
//     }
//     default fn deserialize_map_value(
//         &self,
//         _: &mut dyn erased_serde::de::MapAccess,
//     ) -> Result<Box<dyn ComponentValueBase>, erased_serde::Error> {
//         panic!("Component '{}' is not an extended component", self.get_index())
//     }
//     default fn debug_value(&self, _value: &Box<dyn ComponentValueBase>) -> String {
//         panic!("Component '{}' is not an extended component", self.get_index())
//     }
//     default fn is_extended(&self) -> bool {
//         false
//     }
//     default fn value_to_json_value(&self, _value: &Box<dyn ComponentValueBase>) -> serde_json::Value {
//         panic!("Component '{}' is not an extended component", self.get_index())
//     }
//     default fn value_from_json_value(&self, _value: serde_json::Value) -> Result<Box<dyn ComponentValueBase>, serde_json::Error> {
//         panic!("Component '{}' is not an extended component", self.get_index())
//     }
// }

pub trait IComponentBuffer: Send + Sync {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn desc(&self) -> ComponentDesc;
    fn append(&mut self, buffer: Box<dyn IComponentBuffer>);
    fn push(&mut self, entry: ComponentEntry);
    fn append_cloned(&mut self, entry: ComponentEntry, n: usize);

    fn set(&mut self, index: usize, entry: ComponentEntry) -> ComponentEntry;

    fn swap_remove_index(&mut self, index: usize) -> ComponentEntry;
    fn remove_index(&mut self, index: usize) -> ComponentEntry;

    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;

    fn write_to_world(self: Box<Self>, world: &mut World, entity: EntityId) -> Result<(), ECSError>;
    fn clone_boxed(&self) -> Box<dyn IComponentBuffer>;
    fn clone_value_boxed(&self, index: usize) -> ComponentEntry;
    fn pop(&mut self) -> ComponentEntry;
    fn dump_index(&self, index: usize) -> String;
}

#[derive(Debug, Clone)]
pub struct ComponentBuffer<T: ComponentValue> {
    pub component: crate::component2::Component<T>,
    pub data: Vec<T>,
}

impl<T: ComponentValue> ComponentBuffer<T> {
    pub fn new(component: crate::component2::Component<T>) -> Self {
        Self { component, data: Vec::new() }
    }
    pub fn new_with_value(component: crate::component2::Component<T>, value: T) -> Self {
        Self { component, data: vec![value] }
    }
}

impl<T: ComponentValue + Clone> IComponentBuffer for ComponentBuffer<T> {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn desc(&self) -> ComponentDesc {
        self.component.desc()
    }

    fn append(&mut self, mut buffer: Box<dyn IComponentBuffer>) {
        let b = buffer.as_mut_any().downcast_mut::<ComponentBuffer<T>>().unwrap();
        let x = b.data.pop().unwrap();
        self.data.append(&mut b.data);
        // self.data.resize(self.data.len() + count, x);
    }

    fn push(&mut self, entry: ComponentEntry) {
        self.data.push(entry.into_inner())
    }

    fn append_cloned(&mut self, entry: ComponentEntry, n: usize) {
        self.data.resize(self.data.len() + n, entry.into_inner())
    }

    fn set(&mut self, index: usize, value: ComponentEntry) -> ComponentEntry {
        let b = value.into_inner();
        let old = std::mem::replace(&mut self.data[index], b);
        ComponentEntry::new(self.component, old)
    }

    fn swap_remove_index(&mut self, index: usize) -> ComponentEntry {
        let value = self.data.swap_remove(index);
        ComponentEntry::new(self.component, value)
    }

    fn remove_index(&mut self, index: usize) -> ComponentEntry {
        let value = self.data.remove(index);
        ComponentEntry::new(self.component, value)
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

    fn clone_value_boxed(&self, index: usize) -> ComponentEntry {
        ComponentEntry::new(self.component, self.data[index].clone())
    }

    fn pop(&mut self) -> ComponentEntry {
        ComponentEntry::new(self.component, self.data.pop().unwrap())
    }

    fn dump_index(&self, index: usize) -> String {
        format!("{:?}", self.component.as_debug(&self.data[index]))
    }
}
