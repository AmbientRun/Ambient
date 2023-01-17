use std::fmt;

use itertools::Itertools;
use serde::{
    de::{MapAccess, Visitor}, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer
};

use crate::{
    component2::Serializable, dont_store, query, CreateResources, DeserEntityDataWithWarnings, EntityData, EntityId, World
};

impl Serialize for World {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.len();

        let mut entities = serializer.serialize_map(Some(len))?;
        for (id, _) in query(()).excl(dont_store()).iter(self, None) {
            entities.serialize_entry(&id, &SerWorldEntity { world: self, id })?;
        }
        entities.end()
    }
}
struct SerWorldEntity<'a> {
    world: &'a World,
    id: EntityId,
}
impl<'a> Serialize for SerWorldEntity<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let comps =
            self.world.get_components(self.id).unwrap().into_iter().filter(|x| x.attribute::<Serializable>().is_some()).collect_vec();

        let mut entity = serializer.serialize_map(Some(comps.len()))?;
        for comp in comps {
            if let Some(ser) = comp.attribute::<Serializable>() {
                let value = self.world.get_entry(self.id, comp).unwrap();
                entity.serialize_entry(&comp.path(), ser.serialize(&value))?;
            }
        }
        entity.end()
    }
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct WorldVisitor;

        impl<'de> Visitor<'de> for WorldVisitor {
            type Value = World;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct World")
            }

            fn visit_map<V>(self, mut map: V) -> Result<World, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut res = World::new_with_config_internal("deserialized-world", 0, CreateResources::None);
                while let Some((id, entity)) = map.next_entry::<EntityId, EntityData>()? {
                    res.spawn_mirrored(id, entity);
                }
                Ok(res)
            }
        }

        deserializer.deserialize_map(WorldVisitor)
    }
}

/// Use this struct while de-serializing a World to also get warnings
/// about missing/bad components. Only works with json.
pub struct DeserWorldWithWarnings {
    pub world: World,
    pub warnings: ECSDeserializationWarnings,
}

impl<'de> Deserialize<'de> for DeserWorldWithWarnings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserWorldWithWarningsVisitor;

        impl<'de> Visitor<'de> for DeserWorldWithWarningsVisitor {
            type Value = DeserWorldWithWarnings;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct World")
            }

            fn visit_map<V>(self, mut map: V) -> Result<DeserWorldWithWarnings, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut res = DeserWorldWithWarnings {
                    world: World::new_with_config_internal("deserialized", 0, CreateResources::None),
                    warnings: Default::default(),
                };
                while let Some((id, entity)) = map.next_entry::<EntityId, DeserEntityDataWithWarnings>()? {
                    res.world.spawn_mirrored(id, entity.entity);
                    res.warnings.warnings.extend(entity.warnings.warnings.into_iter().map(|(_, key, err)| (id, key, err)));
                }
                Ok(res)
            }
        }

        deserializer.deserialize_map(DeserWorldWithWarningsVisitor)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ECSDeserializationWarnings {
    pub warnings: Vec<(EntityId, String, String)>,
}

impl std::ops::DerefMut for ECSDeserializationWarnings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.warnings
    }
}

impl std::ops::Deref for ECSDeserializationWarnings {
    type Target = Vec<(EntityId, String, String)>;

    fn deref(&self) -> &Self::Target {
        &self.warnings
    }
}
impl ECSDeserializationWarnings {
    pub fn log_warnings(&self) {
        if !self.warnings.is_empty() {
            log::warn!("{} component bad format errors", self.warnings.len());
            for (id, comp, err) in self.warnings.iter().take(10) {
                log::warn!("Bad component format {} {}: {}", id, comp, err);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{serialization::DeserWorldWithWarnings, *};

    components!("test", {
        @[Serializable]
        ser_test3: String,
        @[Serializable]
        ser_test4: String,
    });

    fn init() {
        crate::init_components();
        init_components();
    }

    #[test]
    pub fn test_serialize_world() {
        init();
        let mut world = World::new("test");
        let id = EntityData::new().set(ser_test3(), "hi".to_string()).spawn(&mut world);

        let ser = serde_json::to_string(&world).unwrap();
        assert_eq!(&ser, r#"{"0:0:0":{},"0:1:0":{"core::test::ser_test3":"hi"}}"#);

        let deser: DeserWorldWithWarnings = serde_json::from_str(&ser).unwrap();
        assert_eq!(deser.world.get_ref(id, ser_test3()).unwrap(), "hi");

        let deser: World = serde_json::from_str(&ser).unwrap();
        assert_eq!(deser.get_ref(id, ser_test3()).unwrap(), "hi");
    }

    #[test]
    pub fn test_serialize_world_resources() {
        init();
        let mut world = World::new("test");
        world.add_resource(ser_test3(), "hi".to_string());
        let ser = serde_json::to_string(&world).unwrap();
        assert_eq!(&ser, r#"{"0:0:0":{"core::test::ser_test3":"hi"}}"#);
        let deser: World = serde_json::from_str(&ser).unwrap();
        assert_eq!(deser.resource(ser_test3()), "hi");
    }

    #[test]
    pub fn test_serialize_world_without_resources() {
        init();
        let world = World::new_with_config("test", 0, false);
        let ser = serde_json::to_string(&world).unwrap();
        assert_eq!(&ser, r#"{}"#);
        let deser: World = serde_json::from_str(&ser).unwrap();
        assert!(!deser.exists(deser.resource_entity()));
    }

    #[test]
    pub fn test_deserialize_bad_world() {
        init();
        let source = r#"{"0:0:0":{},"0:1:0":{"core::test::ser_test3":{"bad":3},"missing":{"hi":5},"core::test::ser_test4":"hello"}}"#;

        let deser: DeserWorldWithWarnings = serde_json::from_str(source).unwrap();
        assert_eq!(deser.world.get_ref(EntityId::new(0, 1, 0), ser_test4()).unwrap(), "hello");
        assert_eq!(deser.warnings.warnings.len(), 2);
        let ser = serde_json::to_string(&deser.world).unwrap();
        assert_eq!(&ser, r#"{"0:0:0":{},"0:1:0":{"core::test::ser_test4":"hello"}}"#);

        assert!(serde_json::from_str::<World>(source).is_err());
    }
}
