use std::{self, any::Any};

use serde::{de::DeserializeOwned, Deserializer, Serializer};

use super::*;
use crate::ComponentEntry;

/// ExComponentValues support serilization, cloning, debug
pub trait ExComponentValue:
    ComponentValue + Serialize + DeserializeOwned + Clone + std::fmt::Debug
{
}
impl<T: ComponentValue + Serialize + DeserializeOwned + Clone + std::fmt::Debug> ExComponentValue
    for T
{
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

    fn write_to_world(self: Box<Self>, world: &mut World, entity: EntityId)
        -> Result<(), ECSError>;
    fn clone_boxed(&self) -> Box<dyn IComponentBuffer>;
    fn clone_value_boxed(&self, index: usize) -> ComponentEntry;
    fn pop(&mut self) -> ComponentEntry;
    fn dump_index(&self, index: usize) -> String;
}

#[derive(Debug, Clone)]
pub struct ComponentBuffer<T: ComponentValue> {
    pub component: crate::Component<T>,
    pub data: Vec<T>,
}

impl<T: ComponentValue> ComponentBuffer<T> {
    pub fn new(component: crate::Component<T>) -> Self {
        Self {
            component,
            data: Vec::new(),
        }
    }
    pub fn new_with_value(component: crate::Component<T>, value: T) -> Self {
        Self {
            component,
            data: vec![value],
        }
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
        let b = buffer
            .as_mut_any()
            .downcast_mut::<ComponentBuffer<T>>()
            .unwrap();
        self.data.append(&mut b.data);
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

    fn write_to_world(
        mut self: Box<Self>,
        world: &mut World,
        entity: EntityId,
    ) -> Result<(), ECSError> {
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
