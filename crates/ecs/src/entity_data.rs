use std::{
    self, fmt::{self, Debug}, iter::Flatten
};

use elements_std::sparse_vec::SparseVec;
use itertools::Itertools;
use serde::{
    de::{MapAccess, Visitor}, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer
};

use super::{with_component_registry, Component, ComponentUnit, ComponentValue, ComponentValueBase, ECSError, EntityId, IComponent, World};
use crate::{ComponentSet, ECSDeserializationWarnings};

#[derive(Clone)]
pub struct EntityData {
    content: SparseVec<ComponentUnit>,
    pub(super) active_components: ComponentSet,
}
impl EntityData {
    pub fn new() -> Self {
        Self { content: SparseVec::new(), active_components: ComponentSet::new() }
    }
    pub fn get<T: Copy + ComponentValue>(&self, component: Component<T>) -> Option<T> {
        self.get_ref(component).copied()
    }
    pub fn get_cloned<T: Clone + ComponentValue>(&self, component: Component<T>) -> Option<T> {
        self.get_ref(component).cloned()
    }
    pub fn get_ref<T: ComponentValue>(&self, component: Component<T>) -> Option<&T> {
        if let Some(unit) = self.content.get(component.get_index()) {
            Some(unit.downcast_value_ref::<T>().expect("Invalid type"))
        } else {
            None
        }
    }
    pub fn get_mut<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T> {
        if let Some(unit) = self.content.get_mut(component.get_index()) {
            Some(unit.downcast_value_mut::<T>().expect("Invalid type"))
        } else {
            None
        }
    }

    pub fn contains<T: ComponentValue>(&self, component: Component<T>) -> bool {
        self.get_ref(component).is_some()
    }
    pub fn set_unit(&mut self, unit: ComponentUnit) {
        self.active_components.insert(unit.component());
        self.content.set(unit.component().get_index(), unit);
    }
    pub fn set_raw(&mut self, component: Box<dyn IComponent>, value: Box<dyn ComponentValueBase>) {
        self.set_unit(ComponentUnit::new_raw(component, value))
    }
    pub fn set_self<T: ComponentValue>(&mut self, component: Component<T>, value: T) {
        let index = component.get_index();
        self.content.set(index, ComponentUnit::new(component, value));
        self.active_components.insert(&component);
    }

    pub fn set<T: ComponentValue>(mut self, component: Component<T>, value: T) -> Self {
        self.set_self(component, value);
        self
    }

    pub fn set_opt<T: ComponentValue>(mut self, component: Component<T>, value: Option<T>) -> Self {
        if let Some(value) = value {
            self.set_self(component, value);
        }
        self
    }

    pub fn set_default<T: Default + ComponentValue>(self, component: Component<T>) -> Self {
        self.set(component, T::default())
    }

    pub fn set_if_empty<T: ComponentValue>(mut self, component: Component<T>, value: T) -> Self {
        if !self.contains(component) {
            self.set_self(component, value);
        }
        self
    }

    pub fn set_default_if_empty<T: Default + ComponentValue>(mut self, component: Component<T>) -> Self {
        if !self.contains(component) {
            self.set_self(component, T::default());
        }
        self
    }

    pub fn remove_raw(&mut self, component: &dyn IComponent) -> Option<ComponentUnit> {
        let value = self.content.remove(component.get_index());
        if value.is_some() {
            self.active_components.remove(component);
        }
        value
    }

    pub fn remove_self<T: ComponentValue>(&mut self, component: Component<T>) -> Option<T> {
        self.remove_raw(&component)?.downcast_value()
    }
    pub fn remove<T: ComponentValue>(mut self, component: Component<T>) -> Self {
        self.remove_self(component);
        self
    }
    pub fn append(mut self, other: EntityData) -> EntityData {
        self.append_self(other);
        self
    }
    pub fn append_self(&mut self, other: EntityData) {
        let other = other.content;
        for unit in other.into_iter() {
            self.set_unit(unit);
        }
    }

    pub fn components(&self) -> Vec<Box<dyn IComponent>> {
        self.content.iter().map(|x| x.component().clone_boxed()).collect_vec()
    }

    pub fn spawn(self, world: &mut World) -> EntityId {
        world.spawn(self)
    }

    pub fn write_to_entity(self, world: &World, entity: EntityId) -> Result<(), ECSError> {
        // TODO: If the new props don't fit the arch of the entity, it needs to be moved
        if let Some(loc) = world.locs.get(entity) {
            world.inc_version();
            let arch = &world.archetypes[loc.archetype];
            arch.write(entity, loc.index, self, world.version());
            Ok(())
        } else {
            Err(ECSError::NoSuchEntity { entity_id: entity })
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentUnit> {
        self.content.iter()
    }

    pub fn filter(&mut self, filter: &dyn Fn(&dyn IComponent) -> bool) {
        let comps = self.components();
        for comp in comps {
            if !filter(comp.as_ref()) {
                self.remove_raw(comp.as_ref());
            }
        }
    }
    pub fn len(&self) -> usize {
        self.content.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl Default for EntityData {
    fn default() -> Self {
        Self::new()
    }
}
impl Debug for EntityData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("EntityData");
        for comp in self.content.iter() {
            out.field(&comp.component().get_id(), &comp.debug_value());
        }
        out.finish()
    }
}

impl Serialize for EntityData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.content.iter().filter(|v| v.component().is_extended()).count();
        let idx_to_id = with_component_registry(|r| r.idx_to_id().clone());

        let mut map = serializer.serialize_map(Some(len))?;
        for unit in self.content.iter() {
            if unit.component().is_extended() {
                let value = unit.value();
                let value = unit.component().serialize_value(&**value);
                map.serialize_entry(idx_to_id.get(&unit.component().get_index()).unwrap(), &value)
                    .expect("Bincode does not support #[serde(flatten)]");
            }
        }
        map.end()
    }
}
impl<'de> Deserialize<'de> for EntityData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntityDataVisitor;

        impl<'de> Visitor<'de> for EntityDataVisitor {
            type Value = EntityData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct EntityData")
            }

            fn visit_map<V>(self, map: V) -> Result<EntityData, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut res = EntityData::new();
                let mut map = erased_serde::de::erase::MapAccess { state: map };
                while let Some(key) = map.state.next_key::<String>()? {
                    let comp = {
                        let comp = with_component_registry(|r| Some(r.get_by_id(&key)?.clone_boxed()));
                        match comp {
                            Some(comp) => comp,
                            None => {
                                log::error!("No such component: {}", key);
                                continue;
                            }
                        }
                    };
                    let value = comp.deserialize_map_value(&mut map).map_err(erased_serde::de::unerase)?;
                    res.set_raw(comp, value);
                }
                Ok(res)
            }
        }

        deserializer.deserialize_map(EntityDataVisitor)
    }
}
/// Use this struct while de-serializing an EntityData to also get warnings
/// about missing/bad components. Only works with serde_json
pub struct DeserEntityDataWithWarnings {
    pub entity: EntityData,
    pub warnings: ECSDeserializationWarnings,
}
impl<'de> Deserialize<'de> for DeserEntityDataWithWarnings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntityDataVisitor;

        impl<'de> Visitor<'de> for EntityDataVisitor {
            type Value = DeserEntityDataWithWarnings;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct EntityData")
            }

            fn visit_map<V>(self, mut map: V) -> Result<DeserEntityDataWithWarnings, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut entity = EntityData::new();
                let mut warnings = ECSDeserializationWarnings::default();
                while let Some((key, value)) = map.next_entry::<serde_json::Value, serde_json::Value>()? {
                    let unit = serde_json::from_value::<ComponentUnit>(serde_json::Value::Array(vec![key.clone(), value]));
                    match unit {
                        Ok(value) => entity.set_unit(value),
                        Err(err) => {
                            let comp = if let serde_json::Value::String(val) = key { val } else { format!("{}", key) };
                            warnings.warnings.push((EntityId::null(), comp, err.to_string()))
                        }
                    }
                }
                Ok(DeserEntityDataWithWarnings { entity, warnings })
            }
        }

        deserializer.deserialize_map(EntityDataVisitor)
    }
}
impl IntoIterator for EntityData {
    type Item = ComponentUnit;

    type IntoIter = Flatten<std::vec::IntoIter<Option<Self::Item>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}
impl FromIterator<ComponentUnit> for EntityData {
    fn from_iter<I: IntoIterator<Item = ComponentUnit>>(iter: I) -> Self {
        let mut c = EntityData::new();

        for i in iter {
            c.set_unit(i);
        }

        c
    }
}

#[cfg(test)]
mod test {
    use crate::{components, ComponentRegistry, EntityData};

    components!("test", {
        ser_test2: String,
    });

    #[test]
    pub fn test_serialize_entity_data() {
        init_components();
        let source = EntityData::new().set(ser_test2(), "hello".to_string());
        let ser = serde_json::to_string(&source).unwrap();
        assert_eq!(&ser, "{\"core::test::ser_test2\":\"hello\"}");
        let deser: EntityData = serde_json::from_str(&ser).unwrap();
        assert_eq!(source.get_ref(ser_test2()), deser.get_ref(ser_test2()));
    }
}
