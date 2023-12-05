use std::time::SystemTime;

use ambient_api::{
    core::{messages::Frame, player::components::user_id},
    element::{use_entity_component, use_module_message, use_query, use_state},
    prelude::*,
};
use ambient_brand_theme::{
    design_tokens::BRANDLIGHT::SEMANTIC_MAIN_ELEMENTS_PRIMARY, window_style, AmbientInternalStyle,
    SEMANTIC_MAIN_ELEMENTS_TERTIARY,
};
use packages::this::{
    components::{
        input_frequency, input_lag, input_timestamp, last_processed_timestamp, local_lag,
        smoothing_factor,
    },
    messages::{Input, ShowInputLagWindow},
};

pub mod packages;

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

    InputLagWindow::el().spawn_interactive();
}

#[element_component]
fn InputLagWindow(hooks: &mut Hooks) -> Element {
    let (visible, set_visible) = use_state(hooks, false);
    use_module_message::<ShowInputLagWindow>(hooks, {
        let set_visible = set_visible.clone();
        move |_, _, _| {
            set_visible(true);
        }
    });

    let lag = use_entity_component(hooks, entity::resources(), local_lag()).unwrap_or_default();

    let close = cb(move || set_visible(false));
    Window {
        title: format!("Input Lag: {:?}", lag),
        visible,
        close: Some(close),
        style: Some(window_style()),
        child: InputLagWindowInner::el(),
    }
    .el()
    .with(min_width(), 200.)
}

#[element_component]
fn InputLagWindowInner(hooks: &mut Hooks) -> Element {
    let mut player_latencies = use_query(hooks, (user_id(), input_lag()));
    player_latencies.sort_unstable();

    let local_player_id = player::get_local();
    let mut players = Vec::with_capacity(player_latencies.len());
    let mut latencies = Vec::with_capacity(player_latencies.len());
    for (id, (player, latency)) in player_latencies.into_iter() {
        let color = if id == local_player_id {
            SEMANTIC_MAIN_ELEMENTS_PRIMARY
        } else {
            SEMANTIC_MAIN_ELEMENTS_TERTIARY
        };
        players.push(Text::el(player).mono_xs_500upp().hex_text_color(color));
        latencies.push(
            Text::el(format!("{:?}", latency))
                .mono_xs_500upp()
                .hex_text_color(color),
        );
    }
    FlowRow::el([
        FlowColumn::el(players).with(space_between_items(), 4.0),
        FlowColumn::el(latencies).with(space_between_items(), 4.0),
    ])
    .with(space_between_items(), 4.0)
}

fn unix_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}
