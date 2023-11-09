use std::time::SystemTime;

use ambient_api::{core::messages::Frame, prelude::*};
use packages::this::{
    components::{
        input_frequency, input_timestamp, last_processed_timestamp, local_lag, smoothing_factor,
    },
    messages::Input,
};

fn unix_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

#[main]
pub fn main() {
    let resources = entity::resources();
    let player_id = player::get_local();

    entity::add_components(
        resources,
        Entity::new()
            .with(input_frequency(), Duration::from_secs(1))
            .with(local_lag(), Duration::ZERO)
            .with(last_processed_timestamp(), Duration::ZERO),
    );

    run_async(async move {
        loop {
            let lag = entity::get_component(resources, local_lag()).unwrap_or_default();
            let timestamp = unix_timestamp();
            Input { timestamp, lag }.send_server_unreliable();
            if let Some(duration) = entity::get_component(resources, input_frequency()) {
                sleep(duration.as_secs_f32()).await;
            } else {
                return;
            }
        }
    });

    Frame::subscribe(move |_| {
        let now = unix_timestamp();
        let last_processed = entity::get_component(resources, last_processed_timestamp())
            .expect("This should have been set up");
        let Some(last_input) = entity::get_component(player_id, input_timestamp()) else {
            // nothing set by the server yet -> skip
            return;
        };
        if last_processed == last_input {
            // already processed -> skip
            return;
        }
        // got a fresh one, mark as processed and process
        entity::set_component(resources, last_processed_timestamp(), last_input);
        let lag = now.saturating_sub(last_input);
        let factor = entity::get_component(resources, smoothing_factor()).unwrap_or(8);
        entity::mutate_component(resources, local_lag(), |old_lag| {
            *old_lag = ((factor - 1) * *old_lag + lag) / factor;
        });
    });
}
