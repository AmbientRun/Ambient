// NOTE(mithun): see explanation in elements/ecs/src/component.rs
#![allow(clippy::borrowed_box)]
use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor}, ser::SerializeTuple, Deserializer, Serializer
};

use super::*;

/// A component + value
pub struct ComponentUnit {
    component: Box<dyn IComponent>,
    value: Box<dyn ComponentValueBase>,
}

impl ComponentUnit {
    pub fn new<T: ComponentValue>(component: Component<T>, value: T) -> Self {
        Self { component: Box::new(component), value: Box::new(value) }
    }
    pub fn new_raw(component: Box<dyn IComponent>, value: Box<dyn ComponentValueBase>) -> Self {
        #[cfg(debug_assertions)]
        if !component.is_valid_value(&value) {
            panic!("Value {} is not valid for component '{}'", value.type_name(), component.get_index());
        }
        Self { component, value }
    }
    pub fn from_entity(world: &World, entity: EntityId, component: &dyn IComponent) -> Result<Self, ECSError> {
        component.clone_value_from_world(world, entity).map(|value| Self::new_raw(component.clone_boxed(), value))
    }
    pub fn component(&self) -> &dyn IComponent {
        self.component.as_ref()
    }
    pub fn into_parts(self) -> (Box<dyn IComponent>, Box<dyn ComponentValueBase>) {
        (self.component, self.value)
    }
    pub fn value(&self) -> &Box<dyn ComponentValueBase> {
        &self.value
    }
    pub fn as_component_value<T: ComponentValue + 'static>(&self, component: Component<T>) -> Option<&T> {
        if self.component() == &component as &dyn IComponent {
            self.downcast_value_ref()
        } else {
            None
        }
    }
    /// Consumes the component and returns the component value
    pub fn downcast_value<T: ComponentValue>(self) -> Option<T> {
        self.value.downcast().ok().map(|v| *v)
    }

    pub fn downcast_value_ref<T: ComponentValue>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }
    pub fn downcast_value_mut<T: ComponentValue>(&mut self) -> Option<&mut T> {
        self.value.downcast_mut()
    }
    pub fn debug_value(&self) -> String {
        self.component.debug_value(&self.value)
    }
    pub fn to_buf(&self) -> Box<dyn IComponentBuffer> {
        self.component.create_buffer_with_value(self.value())
    }
    pub fn set_at_entity(&self, world: &mut World, entity: EntityId) -> Result<ComponentUnit, ECSError> {
        Ok(Self::new_raw(self.component.clone_boxed(), self.component.set_at_entity(world, entity, &self.value)?))
    }
    pub fn add_component_to_entity(&self, world: &mut World, entity: EntityId) -> Result<(), ECSError> {
        self.component.add_component_to_entity(world, entity, &self.value)
    }
    pub fn to_json_value(&self) -> serde_json::Value {
        self.component.value_to_json_value(&self.value)
    }
    pub fn from_json_value(component: Box<dyn IComponent>, value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let value = component.value_from_json_value(value)?;
        Ok(Self::new_raw(component, value))
    }

    pub fn get_component_index(&self) -> usize {
        self.component.get_index()
    }
}
impl Clone for ComponentUnit {
    fn clone(&self) -> Self {
        Self { component: self.component.clone(), value: self.component.clone_value(&self.value) }
    }
}
impl std::fmt::Debug for ComponentUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentUnit").field("component", &self.component).field("value", &self.debug_value()).finish()
    }
}
impl Serialize for ComponentUnit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_tuple(2)?;
        map.serialize_element(&self.component)?;
        let value = self.component.serialize_value(&*self.value);
        map.serialize_element(&value)?;
        map.end()
    }
}
impl<'de> Deserialize<'de> for ComponentUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ComponentUnitVisitor;

        impl<'de> Visitor<'de> for ComponentUnitVisitor {
            type Value = ComponentUnit;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ComponentUnit")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ComponentUnit, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let component: Box<dyn IComponent> = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let mut seq_erased = erased_serde::de::erase::SeqAccess { state: seq };
                let value = component.deserialize_seq_value(&mut seq_erased).map_err(erased_serde::de::unerase)?.unwrap();
                Ok(ComponentUnit { component, value })
            }
        }

        deserializer.deserialize_tuple(2, ComponentUnitVisitor)
    }
}
impl From<Vec<ComponentUnit>> for EntityData {
    fn from(units: Vec<ComponentUnit>) -> Self {
        let mut data = EntityData::new();
        for unit in units.into_iter() {
            data.set_unit(unit);
        }
        data
    }
}
impl From<EntityData> for Vec<ComponentUnit> {
    fn from(ed: EntityData) -> Self {
        ed.iter().cloned().collect_vec()
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    components!("test", {
        ser_test: String,
    });

    fn init() {
        init_components();
    }

    #[test]
    pub fn test_serialize_component() {
        init();
        let source = ser_test();
        let ser = serde_json::to_string(&source).unwrap();
        assert_eq!(&ser, "\"core::test::ser_test\"");
        let deser: Component<String> = serde_json::from_str(&ser).unwrap();
        assert_eq!(source, deser);
    }

    #[test]
    pub fn test() {
        init();
        let source = ComponentUnit::new(ser_test(), "hello".to_string());
        let ser = serde_json::to_string(&source).unwrap();
        assert_eq!(&ser, "[\"core::test::ser_test\",\"hello\"]");
        let deser: ComponentUnit = serde_json::from_str(&ser).unwrap();
        assert_eq!(source.downcast_value_ref::<String>(), deser.downcast_value_ref::<String>());
    }

    #[test]
    pub fn test_json_valule() {
        init();
        let source = ComponentUnit::new(ser_test(), "hello".to_string());
        let value = source.to_json_value();
        let deser: ComponentUnit = ComponentUnit::from_json_value(source.component().clone_boxed(), value).unwrap();
        assert_eq!(source.downcast_value_ref::<String>(), deser.downcast_value_ref::<String>());
    }
}
