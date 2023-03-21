use crate::shared::{implementation::unsupported, wit};

use super::Bindings;

impl wit::client_message::Host for Bindings {
    fn send(
        &mut self,
        _: wit::client_message::Target,
        _: String,
        _: Vec<u8>,
    ) -> anyhow::Result<()> {
        unsupported()
    }
}
