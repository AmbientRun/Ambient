use std::collections::HashMap;

use ambient_ecs::{
    with_component_registry, ComponentDesc, ComponentEntry, Entity, EntityId, Serializable,
    WorldChange, WorldDiff, WorldDiffView,
};
use bincode::Options;
use bytes::Bytes;
use serde::{
    ser::{SerializeSeq, SerializeTuple},
    Deserialize, Deserializer,
};

/// Bincode options used for diff serialization
pub fn bincode_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .allow_trailing_bytes()
}

#[derive(Clone, Debug, Default)]
pub struct WorldDiffDeduplicator {
    last_diff: HashMap<(EntityId, u32), Bytes>,
}

impl WorldDiffDeduplicator {
    pub fn deduplicate<'a>(&mut self, mut diff: WorldDiffView<'a>) -> WorldDiffView<'a> {
        let mut new_diff = HashMap::new();
        diff.changes.retain_mut(|change| {
            // check if we should keep the change and what to drop
            let (keep, components_to_drop) =
                if let WorldChange::SetComponents(id, entity) = change.as_ref() {
                    let mut duplicates = Vec::new();
                    for entry in entity.iter() {
                        let key = (*id, entry.desc().index());
                        // currently comparing serialized bytes since we don't have cmp for components, could be improved
                        let ser = entry
                            .attribute::<Serializable>()
                            .expect("diff should only have serializable components");
                        let bytes: Bytes = bincode_options()
                            .serialize(ser.serialize(entry))
                            .unwrap()
                            .into();
                        new_diff.insert(key, bytes.clone());
                        if self.last_diff.get(&key) == Some(&bytes) {
                            duplicates.push(entry.desc());
                        }
                    }
                    if duplicates.len() == entity.len() {
                        // everything is duplicated -> drop
                        (false, Vec::new())
                    } else {
                        // not all components are duplicated -> drop them but keep the change
                        // NOTE: duplicates can be empty
                        (true, duplicates)
                    }
                } else {
                    // not a SetComponents change -> keep
                    (true, Vec::new())
                };
            if keep && !components_to_drop.is_empty() {
                // we are keeping the entity but there are some components to remove
                if let &mut WorldChange::SetComponents(_, ref mut entity) = change.to_mut() {
                    for component in components_to_drop {
                        entity.remove_raw(component).unwrap();
                    }
                } else {
                    // we only populate components_to_drop for SetComponents changes
                    unreachable!();
                }
            }
            keep
        });
        self.last_diff = new_diff;
        diff
    }
}

/// Explicit tag used for WorldChange enum on the wire
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum WorldChangeTag {
    Spawn = 0,
    Despawn = 1,
    AddComponents = 2,
    RemoveComponents = 3,
    SetComponents = 4,
}

impl From<&WorldChange> for WorldChangeTag {
    fn from(change: &WorldChange) -> Self {
        match change {
            WorldChange::Spawn(_, _) => WorldChangeTag::Spawn,
            WorldChange::Despawn(_) => WorldChangeTag::Despawn,
            WorldChange::AddComponents(_, _) => WorldChangeTag::AddComponents,
            WorldChange::RemoveComponents(_, _) => WorldChangeTag::RemoveComponents,
            WorldChange::SetComponents(_, _) => WorldChangeTag::SetComponents,
        }
    }
}

impl<'a> From<&NetworkedWorldChange<'a>> for WorldChangeTag {
    fn from(change: &NetworkedWorldChange<'a>) -> Self {
        match change {
            NetworkedWorldChange::Spawn(_, _) => WorldChangeTag::Spawn,
            NetworkedWorldChange::Despawn(_) => WorldChangeTag::Despawn,
            NetworkedWorldChange::AddComponents(_, _) => WorldChangeTag::AddComponents,
            NetworkedWorldChange::RemoveComponents(_, _) => WorldChangeTag::RemoveComponents,
            NetworkedWorldChange::SetComponents(_, _) => WorldChangeTag::SetComponents,
        }
    }
}

impl TryFrom<u8> for WorldChangeTag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Spawn),
            1 => Ok(Self::Despawn),
            2 => Ok(Self::AddComponents),
            3 => Ok(Self::RemoveComponents),
            4 => Ok(Self::SetComponents),
            _ => Err(()),
        }
    }
}

impl serde::Serialize for WorldChangeTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

struct WorldChangeTagVisitor;

impl<'de> serde::de::Visitor<'de> for WorldChangeTagVisitor {
    type Value = WorldChangeTag;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("tag for WorldChange enum")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        WorldChangeTag::try_from(v)
            .map_err(|_| serde::de::Error::custom(format!("Unknown WorldChange tag: {:?}", v)))
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for WorldChangeTagVisitor {
    type Value = WorldChangeTag;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(self)
    }
}

impl<'de> serde::Deserialize<'de> for WorldChangeTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(WorldChangeTagVisitor)
    }
}

/// WorldDiff serializer that performs a few optimisations compared to directly serializing with bincode and our text serialization:
/// - indexes component paths and includes the index in the serialized format (path length + 8 bytes -> 4 bytes + extra index when a new component path is encountered)
/// - serializes Entity Ids as u128 instead of base64 encoded strings (29 bytes -> 16 bytes)
/// - uses variable length int encoding to save space on collection lengths (8 bytes -> usually 1 byte)
#[derive(Clone, Debug, Default)]
pub struct DiffSerializer {
    known_component_paths: HashMap<u32, String>,
}

impl DiffSerializer {
    pub fn serialize(&mut self, diff: &WorldDiffView) -> Result<Bytes, bincode::Error> {
        let unknown_components =
            self.collect_unknown_components(diff.changes.iter().map(AsRef::as_ref));
        let mut buffer = bincode_options().serialize(&unknown_components)?;
        self.known_component_paths
            .extend(unknown_components.into_iter());
        buffer.extend_from_slice(&bincode_options().serialize(&NetworkedWorldDiff(diff))?);
        Ok(buffer.into())
    }

    fn collect_unknown_components<'a, I>(&self, changes: I) -> HashMap<u32, String>
    where
        I: Iterator<Item = &'a WorldChange>,
    {
        let mut result = HashMap::new();
        let mut collect = |desc: &ComponentDesc| {
            if !self.known_component_paths.contains_key(&desc.index()) {
                result.insert(desc.index(), desc.path());
            }
        };

        for change in changes {
            match change {
                WorldChange::Spawn(_, entity)
                | WorldChange::AddComponents(_, entity)
                | WorldChange::SetComponents(_, entity) => {
                    for desc in entity.iter().map(|entry| entry.desc()) {
                        collect(&desc);
                    }
                }
                WorldChange::Despawn(_) => {}
                WorldChange::RemoveComponents(_, components) => {
                    for desc in components {
                        collect(desc);
                    }
                }
            }
        }
        result
    }

    pub fn deserialize(&mut self, message: Bytes) -> Result<WorldDiff, bincode::Error> {
        let mut deserializer =
            bincode::Deserializer::with_reader(message.as_ref(), bincode_options());
        let unknown_components = HashMap::<u32, String>::deserialize(&mut deserializer)?;
        self.known_component_paths
            .extend(unknown_components.into_iter());

        deserializer.deserialize_seq(NetworkedChangesVisitor {
            known_component_paths: &self.known_component_paths,
        })
        // bincode_options().deserialize(&message)
    }
}

struct NetworkedChangesVisitor<'a> {
    known_component_paths: &'a HashMap<u32, String>,
}

impl<'a, 'de> serde::de::Visitor<'de> for NetworkedChangesVisitor<'a> {
    type Value = WorldDiff;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct WorldDiff")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut changes = if let Some(size) = seq.size_hint() {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };
        while let Some(change) = seq.next_element_seed(NetworkedChangeVisitor {
            known_component_paths: self.known_component_paths,
        })? {
            changes.push(change)
        }
        Ok(WorldDiff { changes })
    }
}

struct NetworkedChangeVisitor<'a> {
    known_component_paths: &'a HashMap<u32, String>,
}

impl<'a, 'de> serde::de::Visitor<'de> for NetworkedChangeVisitor<'a> {
    type Value = WorldChange;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("enum WorldChange")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // WorldChange is encoded as a tuple of 3 elements:
        // 1. tag
        // 2. id
        // 3. change specific data (either Entity, Vec<ComponentDesc> or 0u8 for Despawn)

        let tag = seq
            .next_element_seed(WorldChangeTagVisitor)?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let id = seq
            .next_element_seed(NetworkedEntityIdVisitor)?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(match tag {
            WorldChangeTag::Spawn => {
                let entity = seq
                    .next_element_seed(NetworkedEntityVisitor {
                        known_component_paths: self.known_component_paths,
                    })?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                WorldChange::Spawn(id, entity)
            }
            WorldChangeTag::Despawn => {
                // here we are dropping the 0 byte for Despawn
                let _ = seq
                    .next_element_seed(WorldChangeTagVisitor)?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                WorldChange::Despawn(id)
            }
            WorldChangeTag::AddComponents => {
                let entity = seq
                    .next_element_seed(NetworkedEntityVisitor {
                        known_component_paths: self.known_component_paths,
                    })?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                WorldChange::AddComponents(id, entity)
            }
            WorldChangeTag::RemoveComponents => {
                let components = seq
                    .next_element_seed(NetworkedComponentDescsVisitor {
                        known_component_paths: self.known_component_paths,
                    })?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                WorldChange::RemoveComponents(id, components)
            }
            WorldChangeTag::SetComponents => {
                let entity = seq
                    .next_element_seed(NetworkedEntityVisitor {
                        known_component_paths: self.known_component_paths,
                    })?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                WorldChange::SetComponents(id, entity)
            }
        })
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for NetworkedChangeVisitor<'a> {
    type Value = WorldChange;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // WorldChange is encoded as a tuple of 3 elements:
        // 1. tag
        // 2. id
        // 3. change specific data (either Entity, Vec<ComponentDesc> or 0u8 for Despawn)
        deserializer.deserialize_tuple(3, self)
    }
}

struct NetworkedEntityIdVisitor;

impl<'de> serde::de::Visitor<'de> for NetworkedEntityIdVisitor {
    type Value = EntityId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("128-bit EntityId")
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(EntityId(v))
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for NetworkedEntityIdVisitor {
    type Value = EntityId;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u128(self)
    }
}

struct NetworkedEntityVisitor<'a> {
    known_component_paths: &'a HashMap<u32, String>,
}

impl<'a, 'de> serde::de::Visitor<'de> for NetworkedEntityVisitor<'a> {
    type Value = Entity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Entity")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut entity = Entity::new();
        while let Some(entry) = seq.next_element_seed(NetworkedComponentEntryVisitor {
            known_component_paths: self.known_component_paths,
        })? {
            entity.set_entry(entry);
        }
        Ok(entity)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut entity = Entity::new();
        while let Some(desc) = map.next_key_seed(NetworkedComponentDescVisitor {
            known_component_paths: self.known_component_paths,
        })? {
            let Some(ser) = desc.attribute::<Serializable>() else {
                return Err(serde::de::Error::custom(format!("tried to deserialize non-serializable component {:?}", desc)));
            };
            let entry = map.next_value_seed(ser.deserializer(desc))?;
            entity.set_entry(entry);
        }
        Ok(entity)
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for NetworkedEntityVisitor<'a> {
    type Value = Entity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

struct NetworkedComponentDescsVisitor<'a> {
    known_component_paths: &'a HashMap<u32, String>,
}

impl<'a, 'de> serde::de::Visitor<'de> for NetworkedComponentDescsVisitor<'a> {
    type Value = Vec<ComponentDesc>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("sequence of networked ComponentDesc")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut components = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        while let Some(desc) = seq.next_element_seed(NetworkedComponentDescVisitor {
            known_component_paths: self.known_component_paths,
        })? {
            components.push(desc);
        }
        Ok(components)
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for NetworkedComponentDescsVisitor<'a> {
    type Value = Vec<ComponentDesc>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

struct NetworkedComponentDescVisitor<'a> {
    known_component_paths: &'a HashMap<u32, String>,
}

impl<'a, 'de> serde::de::Visitor<'de> for NetworkedComponentDescVisitor<'a> {
    type Value = ComponentDesc;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("u32 component index")
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match self.known_component_paths.get(&v) {
            Some(path) => {
                let component = with_component_registry(|r| r.get_by_path(path));
                match component {
                    Some(desc) => Ok(desc),
                    None => Err(serde::de::Error::custom(format!("No such component: {v}"))),
                }
            }
            None => Err(serde::de::Error::custom(format!(
                "Unknown component index {}",
                v
            ))),
        }
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for NetworkedComponentDescVisitor<'a> {
    type Value = ComponentDesc;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(self)
    }
}

struct NetworkedComponentEntryVisitor<'a> {
    known_component_paths: &'a HashMap<u32, String>,
}

impl<'a, 'de> serde::de::Visitor<'de> for NetworkedComponentEntryVisitor<'a> {
    type Value = ComponentEntry;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct ComponentEntry")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let desc = seq
            .next_element_seed(NetworkedComponentDescVisitor {
                known_component_paths: self.known_component_paths,
            })?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let Some(ser) = desc.attribute::<Serializable>() else {
            return Err(serde::de::Error::custom(format!("tried to deserialize non-serializable component {:?}", desc)));
        };
        let entry = seq
            .next_element_seed(ser.deserializer(desc))?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        Ok(entry)
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for NetworkedComponentEntryVisitor<'a> {
    type Value = ComponentEntry;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, self)
    }
}

struct NetworkedWorldDiff<'a>(&'a WorldDiffView<'a>);

impl<'a> serde::Serialize for NetworkedWorldDiff<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.changes.len()))?;
        for change in self.0.changes.iter().map(|change| change.as_ref()) {
            seq.serialize_element(&NetworkedWorldChange::from(change))?;
        }
        seq.end()
    }
}

#[derive(Clone, Debug)]
enum NetworkedWorldChange<'a> {
    Spawn(u128, NetworkedEntity<'a>),
    Despawn(u128),
    AddComponents(u128, NetworkedEntity<'a>),
    RemoveComponents(u128, Vec<NetworkedComponentDesc>),
    SetComponents(u128, NetworkedEntity<'a>),
}

impl<'a> NetworkedWorldChange<'a> {
    fn id(&self) -> u128 {
        match self {
            NetworkedWorldChange::Spawn(id, _)
            | NetworkedWorldChange::Despawn(id)
            | NetworkedWorldChange::AddComponents(id, _)
            | NetworkedWorldChange::RemoveComponents(id, _)
            | NetworkedWorldChange::SetComponents(id, _) => *id,
        }
    }

    fn entity(&self) -> Option<NetworkedEntity<'a>> {
        match self {
            NetworkedWorldChange::Despawn(_) | NetworkedWorldChange::RemoveComponents(_, _) => None,
            NetworkedWorldChange::Spawn(_, e)
            | NetworkedWorldChange::AddComponents(_, e)
            | NetworkedWorldChange::SetComponents(_, e) => Some(*e),
        }
    }
}

impl<'a> From<&'a WorldChange> for NetworkedWorldChange<'a> {
    fn from(value: &'a WorldChange) -> Self {
        match value {
            WorldChange::Spawn(id, entity) => Self::Spawn(id.0, NetworkedEntity(entity)),
            WorldChange::Despawn(id) => Self::Despawn(id.0),
            WorldChange::AddComponents(id, entity) => {
                Self::AddComponents(id.0, NetworkedEntity(entity))
            }
            WorldChange::RemoveComponents(id, components) => Self::RemoveComponents(
                id.0,
                components
                    .iter()
                    .map(|desc| NetworkedComponentDesc(desc.index()))
                    .collect(),
            ),
            WorldChange::SetComponents(id, entity) => {
                Self::SetComponents(id.0, NetworkedEntity(entity))
            }
        }
    }
}

impl<'a> serde::Serialize for NetworkedWorldChange<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // WorldChange is encoded as a tuple of 3 elements:
        // 1. tag
        // 2. id
        // 3. change specific data (either Entity, Vec<ComponentDesc> or 0u8 for Despawn)
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element(&WorldChangeTag::from(self))?;
        seq.serialize_element(&self.id())?;
        if let Some(e) = self.entity() {
            seq.serialize_element(&e)?;
        } else if let NetworkedWorldChange::RemoveComponents(_, components) = self {
            seq.serialize_element(components)?;
        } else if let NetworkedWorldChange::Despawn(_) = self {
            seq.serialize_element(&0u8)?;
        } else {
            unreachable!();
        }
        seq.end()
    }
}

#[derive(Clone, Copy, Debug)]
struct NetworkedEntity<'a>(&'a Entity);

impl<'a> serde::Serialize for NetworkedEntity<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for entry in self.0.iter() {
            seq.serialize_element(&NetworkedComponentEntry(entry))?;
        }
        seq.end()
    }
}

#[derive(Clone, Copy, Debug)]
struct NetworkedComponentEntry<'a>(&'a ComponentEntry);

impl<'a> serde::Serialize for NetworkedComponentEntry<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element(&self.0.desc().index())?;
        let Some(ser) = self.0.desc().attribute::<Serializable>() else {
            return Err(serde::ser::Error::custom(format!("tried to serialize non-serializable component {:?}", self.0)));
        };
        seq.serialize_element(ser.serialize(self.0))?;
        seq.end()
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
struct NetworkedComponentDesc(u32);

#[cfg(test)]
mod tests {
    use ambient_ecs::components;

    use super::*;

    components!("test", {
        @[Serializable]
        text: String,
        @[Serializable]
        float: f32,
        @[Serializable]
        counter: usize,
    });

    fn assert_same_diff(a: &WorldDiff, b: &WorldDiff) {
        assert_eq!(a.changes.len(), b.changes.len());
        for (a, b) in a.changes.iter().zip(b.changes.iter()) {
            assert_eq!(format!("{:?}", a), format!("{:?}", b));
        }
    }

    fn assert_passes_through_serialization(diff: WorldDiff) {
        let view = WorldDiffView::from(&diff);

        let bytes = DiffSerializer::default().serialize(&view).unwrap();
        let received_diff = DiffSerializer::default().deserialize(bytes).unwrap();

        assert_same_diff(&diff, &received_diff);
    }

    #[test]
    fn simple_changes_serialize_correctly() {
        init_components();
        let id = EntityId::new();
        let entity = Entity::new()
            .with(text(), "foo".to_string())
            .with(float(), 1234.567)
            .with(counter(), 42);
        assert_passes_through_serialization(WorldDiff {
            changes: vec![WorldChange::Spawn(id, entity.clone())],
        });
        assert_passes_through_serialization(WorldDiff {
            changes: vec![WorldChange::Despawn(id)],
        });
        assert_passes_through_serialization(WorldDiff {
            changes: vec![WorldChange::SetComponents(id, entity.clone())],
        });
        assert_passes_through_serialization(WorldDiff {
            changes: vec![WorldChange::AddComponents(id, entity.clone())],
        });
        assert_passes_through_serialization(WorldDiff {
            changes: vec![WorldChange::RemoveComponents(id, entity.components())],
        });
        assert_passes_through_serialization(WorldDiff {
            changes: vec![
                WorldChange::Spawn(id, entity.clone()),
                WorldChange::Despawn(id),
                WorldChange::SetComponents(id, entity.clone()),
                WorldChange::AddComponents(id, entity.clone()),
                WorldChange::RemoveComponents(id, entity.components()),
            ],
        });
    }
}
