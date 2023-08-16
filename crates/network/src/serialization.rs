/// A wrapper around a type that can be serialized and deserialized transparently but captures deserialize errors instead of failing.
/// This allows for more graceful handling of errors in the deserialization process and for getting partial results.
#[derive(Clone, Debug)]
pub enum FailableDeserialization<T> {
    Ok(T),
    Error(String),
}

impl<T> FailableDeserialization<T> {
    pub fn ok(self) -> Option<T> {
        match self {
            FailableDeserialization::Ok(t) => Some(t),
            FailableDeserialization::Error(_) => None,
        }
    }
}

impl<T> From<T> for FailableDeserialization<T> {
    fn from(t: T) -> Self {
        FailableDeserialization::Ok(t)
    }
}

impl<T> From<FailableDeserialization<T>> for Result<T, String> {
    fn from(value: FailableDeserialization<T>) -> Self {
        match value {
            FailableDeserialization::Ok(t) => Ok(t),
            FailableDeserialization::Error(e) => Err(e),
        }
    }
}

impl<T> serde::Serialize for FailableDeserialization<T>
where
    T: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            FailableDeserialization::Ok(t) => t.serialize(serializer),
            FailableDeserialization::Error(_) => Err(serde::ser::Error::custom(
                "Cannot serialize FailableDeserialization::Error",
            )),
        }
    }
}

impl<'de, T> serde::Deserialize<'de> for FailableDeserialization<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match T::deserialize(deserializer) {
            Ok(t) => FailableDeserialization::Ok(t),
            Err(e) => FailableDeserialization::Error(format!("{}", e)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_directly() {
        let value = FailableDeserialization::Ok(42);
        let serialized = bincode::serialize(&value).unwrap();
        let directly_serialized = bincode::serialize(&42).unwrap();
        assert_eq!(serialized, directly_serialized);
    }

    #[test]
    fn deserializes() {
        let value: u64 = 42;
        let serialized = bincode::serialize(&value).unwrap();
        let deserialized: FailableDeserialization<u64> = bincode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.ok(), Some(value));
    }

    #[test]
    fn captures_deserialization_errors() {
        let serialized = bincode::serialize(&42).unwrap();
        let deserialized: FailableDeserialization<String> =
            bincode::deserialize(&serialized).unwrap();
        let Err(msg) = deserialized.into() else {
            panic!("Expected deserialization error");
        };
        assert!(!msg.is_empty());
    }
}
