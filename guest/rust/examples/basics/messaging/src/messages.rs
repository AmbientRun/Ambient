use ambient_api::message::{Message, MessageSerde, MessageSerdeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hello {
    pub text: String,
    pub source_reliable: bool,
}
impl Message for Hello {
    fn id() -> &'static str {
        "hello"
    }

    fn serialize_message(&self) -> Result<Vec<u8>, MessageSerdeError> {
        let mut output = vec![];
        self.text.serialize_message_part(&mut output)?;
        self.source_reliable.serialize_message_part(&mut output)?;
        Ok(output)
    }

    fn deserialize_message(mut input: &[u8]) -> Result<Self, MessageSerdeError> {
        Ok(Hello {
            text: String::deserialize_message_part(&mut input)?,
            source_reliable: bool::deserialize_message_part(&mut input)?,
        })
    }
}
