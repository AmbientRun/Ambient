use std::fmt;

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserializer, Serializer,
};

use super::*;
use crate::Serializable;

impl Serialize for ComponentEntry {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_tuple(2)?;
        let ser = self.attribute::<Serializable>().expect("Component is not serializable");

        map.serialize_element(&self.desc())?;
        map.serialize_element(&ser.serialize(self))?;

        map.end()
    }
}

impl<'de> Deserialize<'de> for ComponentEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = ComponentEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ComponentEntry")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ComponentEntry, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let desc: ComponentDesc = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let ser = *desc.attribute::<Serializable>().expect("Component is not serializable");

                let value = seq.next_element_seed(ser.deserializer(desc))?.ok_or_else(|| de::Error::invalid_length(0, &self));
                value
            }
        }

        deserializer.deserialize_tuple(2, EntryVisitor)
    }
}

impl From<Vec<ComponentEntry>> for Entity {
    fn from(entries: Vec<ComponentEntry>) -> Self {
        let mut data = Entity::new();
        for entry in entries.into_iter() {
            data.set_entry(entry);
        }
        data
    }
}
impl From<Entity> for Vec<ComponentEntry> {
    fn from(ed: Entity) -> Self {
        ed.iter().cloned().collect_vec()
    }
}

#[cfg(test)]
mod test {
    use crate::{Serializable, *};

    components!("test", {
        @[Serializable]
        ser_test: String,
    });

    fn init() {
        init_components();
    }

    #[test]
    pub fn test_serialize_component() {
        init();
        let source = ser_test().desc();
        let ser = serde_json::to_string(&source).unwrap();
        assert_eq!(&ser, "\"core::test::ser_test\"");
        let deser: ComponentDesc = serde_json::from_str(&ser).unwrap();
        assert_eq!(source.index(), deser.index());
    }

    #[test]
    pub fn test() {
        init();
        let source = ComponentEntry::new(ser_test(), "hello".to_string());
        let ser = serde_json::to_string(&source).unwrap();
        assert_eq!(&ser, "[\"core::test::ser_test\",\"hello\"]");
        let deser: ComponentEntry = serde_json::from_str(&ser).unwrap();
        assert_eq!(source.downcast_ref::<String>(), deser.downcast_ref::<String>());
    }

    #[test]
    pub fn test_json_value() {
        init();
        let source = ComponentEntry::new(ser_test(), "hello".to_string());
        let value = source.desc().to_json(&source).unwrap();
        let deser: ComponentEntry = source.desc().from_json(&value).unwrap();
        assert_eq!(source.downcast_ref::<String>(), deser.downcast_ref::<String>());
    }
}
