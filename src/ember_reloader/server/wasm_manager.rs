use ambient_api::{
    components::core::wasm::{bytecode_from_url, module_enabled},
    prelude::*,
};

use crate::messages;

pub fn main() {
    messages::WasmSetEnabled::subscribe(|_, msg| {
        entity::set_component(msg.id, module_enabled(), msg.enabled);
    });

    messages::WasmReload::subscribe(|source, msg| {
        let Some(user_id) = source.client_user_id() else { return; };
        let id = msg.id;

        run_async(async move {
            if let Err(err) = asset::build_wasm().await {
                messages::ErrorMessage::new(err.to_string()).send_client_targeted_reliable(user_id);
                return;
            }

            if let Some(url) = entity::get_component(id, bytecode_from_url()) {
                entity::set_component(id, bytecode_from_url(), url);
            }
        });
    });
}
