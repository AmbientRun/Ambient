use ambient_api::prelude::*;

use shared::*;
mod shared;

#[main]
pub async fn main() {
    let console = Console::new(true);

    messages::ConsoleServerInput::subscribe(move |source, msg| {
        let Some(user_id) = source.client_user_id() else { return; };

        console.lock().unwrap().input(&msg.input, {
            let user_id = user_id.clone();
            move |line| {
                messages::ConsoleServerOutput {
                    text: line.text,
                    ty: line.ty.into(),
                    is_server: line.is_server,
                }
                .send_client_targeted_reliable(user_id.clone());
            }
        });
    });
}
