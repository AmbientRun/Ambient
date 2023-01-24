use std::{
    self, fmt::{self, Debug}, iter::Flatten
};

use elements_std::sparse_vec::SparseVec;
use itertools::Itertools;
use serde::{
    de::{self, DeserializeSeed, MapAccess, Visitor}, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer
};

use super::{with_component_registry, Component, ComponentValue, ECSError, EntityId, World};
use crate::{ComponentDesc, ComponentEntry, ComponentSet, ECSDeserializationWarnings, Serializable};

#[derive(Clone)]
pub struct EntityData {
    content: SparseVec<ComponentEntry>,
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
        if let Some(unit) = self.content.get(component.index() as _) {
            Some(unit.downcast_ref::<T>())
        } else {
            None
        }
    }
    pub fn get_mut<T: ComponentValue>(&mut self, component: Component<T>) -> Option<&mut T> {
        if let Some(unit) = self.content.get_mut(component.index() as _) {
            Some(unit.downcast_mut::<T>())
        } else {
            None
        }
    }

    pub fn contains<T: ComponentValue>(&self, component: Component<T>) -> bool {
        self.get_ref(component).is_some()
    }

    pub fn set_entry(&mut self, entry: ComponentEntry) {
        self.active_components.insert(entry.desc());
        self.content.set(entry.desc().index() as _, entry);
    }

    pub fn set_self<T: ComponentValue>(&mut self, component: Component<T>, value: T) {
        let index = component.index() as _;
        self.content.set(index, ComponentEntry::new(component, value));
        self.active_components.insert(component.desc());
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

    pub fn remove_raw(&mut self, desc: ComponentDesc) -> Option<ComponentEntry> {
        let value = self.content.remove(desc.index() as usize);

        if value.is_some() {
            self.active_components.remove(desc);
        }

        value
    }

    pub fn remove_self<T: ComponentValue>(&mut self, component: Component<T>) -> Option<T> {
        Some(self.remove_raw(component.desc())?.into_inner())
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
        for entry in other {
            self.set_entry(entry);
        }
    }

    pub fn components(&self) -> Vec<ComponentDesc> {
        self.content.iter().map(|x| x.desc()).collect_vec()
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

    pub fn iter(&self) -> impl Iterator<Item = &ComponentEntry> {
        self.content.iter()
    }

    pub fn filter(&mut self, filter: &dyn Fn(ComponentDesc) -> bool) {
        let comps = self.components();
        for entry in comps {
            if !filter(entry) {
                self.remove_raw(entry);
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
        for entry in self.content.iter() {
            out.field(&entry.desc().name(), &entry.as_debug());
        }
        out.finish()
    }
}

impl Serialize for EntityData {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.content.iter().filter(|v| v.attribute::<Serializable>().is_some()).count();

        let mut map = serializer.serialize_map(Some(len))?;
        for unit in self.content.iter() {
            if let Some(ser) = unit.attribute::<Serializable>() {
                let value = ser.serialize(unit);
                map.serialize_entry(&unit.desc().path(), &value).expect("Bincode does not support #[serde(flatten)]");
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

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut res = EntityData::new();
                while let Some(key) = map.next_key::<String>()? {
                    let desc = with_component_registry(|r| r.get_by_path(&key))
                        .ok_or_else(|| de::Error::custom(format!("No such component: {key}")))?;

                    let ser = desc
                        .attribute::<Serializable>()
                        .ok_or_else(|| de::Error::custom(format!("Component {desc:?} is not deserializable")))?;

                    let value = map.next_value_seed(*ser)?;

                    res.set_entry(value);
                }

                Ok(res)
            }
        }

        deserializer.deserialize_map(EntityDataVisitor)
    }
}

impl<'de> Deserialize<'de> for DeserEntityDataWithWarnings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntityDataVisitor {
            warnings: ECSDeserializationWarnings,
        }

        impl<'de> Visitor<'de> for EntityDataVisitor {
            type Value = DeserEntityDataWithWarnings;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct EntityData")
            }

            fn visit_map<V>(mut self, map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut res = EntityData::new();
                let mut map = erased_serde::de::erase::MapAccess { state: map };
                while let Some((key, value)) = map.state.next_entry::<String, serde_json::Value>()? {
                    let desc = with_component_registry(|r| r.get_by_path(&key));
                    let desc = match desc {
                        Some(desc) => desc,

                        None => {
                            self.warnings.push((EntityId::null(), key.clone(), format!("No such component: {key}")));
                            continue;
                        }
                    };

                    let ser: Result<_, V::Error> = desc
                        .attribute::<Serializable>()
                        .ok_or_else(|| de::Error::custom(format!("Component {desc:?} is not deserializable")));

                    let ser = match ser {
                        Ok(v) => v,
                        Err(err) => {
                            self.warnings.push((EntityId::null(), key, format!("{err:?}")));
                            continue;
                        }
                    };

                    let value = ser.deserialize(value);
                    let value = match value {
                        Ok(v) => v,
                        Err(err) => {
                            self.warnings.push((EntityId::null(), key, format!("{err:?}")));
                            continue;
                        }
                    };

                    res.set_entry(value);
                }

                Ok(DeserEntityDataWithWarnings { entity: res, warnings: self.warnings })
            }
        }

        deserializer.deserialize_map(EntityDataVisitor { warnings: Default::default() })
    }
}

/// Use this struct while de-serializing an EntityData to also get warnings
/// about missing/bad components. Only works with serde_json
pub struct DeserEntityDataWithWarnings {
    pub entity: EntityData,
    pub warnings: ECSDeserializationWarnings,
}

impl IntoIterator for EntityData {
    type Item = ComponentEntry;

    type IntoIter = Flatten<std::vec::IntoIter<Option<Self::Item>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}
impl FromIterator<ComponentEntry> for EntityData {
    fn from_iter<I: IntoIterator<Item = ComponentEntry>>(iter: I) -> Self {
        let mut c = EntityData::new();

        for v in iter {
            c.set_entry(v);
        }

        c
    }
}

#[cfg(test)]
mod test {
    use crate::{components, EntityData, Networked};

    components!("test", {
        @[Networked]
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
