use ambient_api::{core::app::components::name, prelude::*};
use packages::this::{components::*, concepts::Tuner, messages::UpdateTuner};

#[main]
pub fn main() {
    init_tuners();
    // spawn_test_tuners();
    listen_client();
}

fn init_tuners() {
    change_query((raw_value(), tuner_min(), tuner_max()))
        .track_change((raw_value(), tuner_min(), tuner_max()))
        .bind(|tuners| {
            for (tuner, (raw, tmin, tmax)) in tuners {
                entity::add_component(tuner, output(), tmin + (tmax - tmin) * raw);
            }
        });
}

fn listen_client() {
    UpdateTuner::subscribe(|_ctx, msg| {
        let mut tunername: Option<String> = None;
        let mut prevraw: Option<f32> = None;
        if entity::exists(msg.id) {
            tunername = entity::get_component(msg.id, name());
            prevraw = entity::get_component(msg.id, raw_value());
            entity::add_component(msg.id, raw_value(), msg.raw);
            println!(
                "Changed value of {:?} from {:?} to {}",
                tunername, prevraw, msg.raw
            );
        } else {
            println!(
                "(!) Did not change value of {:?} from {:?} to {}",
                tunername, prevraw, msg.raw
            );
        }
    });
}

#[allow(dead_code)]
fn spawn_test_tuners() {
    Tuner {
        description: "This is a test tuner, it does nothing".to_string(),
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Test Tuner 1".to_string())
    .spawn();
    Tuner {
        description: "This is another test tuner, it also does nothing".to_string(),
        tuner_max: 100.,
        ..Tuner::suggested()
    }
    .make()
    .with(name(), "Test Tuner 2".to_string())
    .spawn();
}
