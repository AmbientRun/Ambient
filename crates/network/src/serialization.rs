/// A wrapper around a type that can be serialized and deserialized transparently but captures deserialize errors
/// instead of failing.
/// This allows for more graceful handling of errors in the deserialization process and for getting partial results.
///
/// Note that when using FailableDeserialization for part of your type then it is supposed to be used for last field(s)
/// of your type as anything deserialized after it will most likely fail.
#[derive(Clone, Debug)]
pub struct FailableDeserialization<T>(Result<T, String>);

impl<T> FailableDeserialization<T> {
    pub fn into_inner(self) -> Result<T, String> {
        self.0
    }
}

impl<T> From<T> for FailableDeserialization<T> {
    fn from(t: T) -> Self {
        Ok(t).into()
    }
}

impl<T> From<Result<T, String>> for FailableDeserialization<T> {
    fn from(result: Result<T, String>) -> Self {
        Self(result)
    }
}

impl<T> serde::Serialize for FailableDeserialization<T>
where
    T: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            Ok(t) => t.serialize(serializer),
            Err(_) => Err(serde::ser::Error::custom(
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
        Ok(T::deserialize(deserializer)
            .map_err(|e| e.to_string())
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_directly() {
        let value = FailableDeserialization(Ok(42));
        let serialized = bincode::serialize(&value).unwrap();
        let directly_serialized = bincode::serialize(&42).unwrap();
        assert_eq!(serialized, directly_serialized);
    }

    #[test]
    fn deserializes() {
        let value: u64 = 42;
        let serialized = bincode::serialize(&value).unwrap();
        let deserialized: FailableDeserialization<u64> = bincode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.into_inner(), Ok(value));
    }

    #[test]
    fn captures_deserialization_errors() {
        let serialized = bincode::serialize(&42).unwrap();
        let deserialized: FailableDeserialization<String> =
            bincode::deserialize(&serialized).unwrap();
        let error_message = deserialized
            .into_inner()
            .expect_err("Expected deserialization error");
        assert!(!error_message.is_empty());
    }
}
