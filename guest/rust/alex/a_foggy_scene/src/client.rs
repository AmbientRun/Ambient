use ambient_api::{
    core::{
        audio::components::amplitude,
        camera::components::{fog, perspective_infinite_reverse},
        rendering::components::{fog_color, fog_density},
    },
    prelude::*,
};
use packages::{temperature::components::temperature, this::components::ambient_loop};

const DEATH_TEMP: f32 = 21.;
const NORMAL_TEMP: f32 = 37.;

#[main]
pub fn main() {
    spawn_query(())
        .requires(perspective_infinite_reverse())
        .bind(|cameras| {
            if let Some((camera, _)) = cameras.into_iter().next() {
                entity::add_component(camera, fog(), ());

                spawn_query(ambient_loop()).bind(move |ambient_loopers| {
                    for (looper, loop_path) in ambient_loopers {
                        let spatial_audio_player = audio::SpatialAudioPlayer::new();
                        spatial_audio_player.set_amplitude(2.0);
                        spatial_audio_player.set_looping(true);
                        spatial_audio_player.set_listener(camera);
                        spatial_audio_player.play_sound_on_entity(loop_path, looper);
                    }
                });
            }
        });

    let sun = make_my_local_sun_with_sky();

    let storm_sound_player = audio::AudioPlayer::new();
    storm_sound_player.set_looping(true);
    storm_sound_player.set_amplitude(0.0);
    let storm_sound_playing =
        storm_sound_player.play(packages::this::assets::url("snowstorm_ambience.ogg"));

    ambient_api::core::messages::Frame::subscribe(move |_| {
        let coldness: f32 = remap32(
            entity::get_component(player::get_local(), temperature()).unwrap_or(1.),
            DEATH_TEMP,
            NORMAL_TEMP,
            1.0,
            0.0,
        );
        entity::mutate_component_with_default(storm_sound_playing, amplitude(), 0.0, |amp| {
            *amp =
                *amp * 0.8 + 0.2 * (0.05 + game_time().as_secs_f32().sin() * 0.05 + coldness * 1.5)
        });
        if coldness < 0.60 {
            let t = coldness / 0.60;
            entity::mutate_component(sun, fog_density(), |foggy| {
                *foggy = *foggy * 0.9 + 0.1 * (0.02 + 0.18 * t);
            });
        } else {
            let t = (coldness - 0.60) / (1. - 0.60);
            entity::set_component(sun, fog_density(), 0.20 + 0.80 * t * t);
        }
        entity::mutate_component(sun, fog_color(), |color| {
            *color = color.lerp(
                vec3(0.75, 0.45, 0.75).lerp(vec3(0.60, 1.00, 1.00), coldness.sqrt()),
                0.1,
            )
        });
    });
}

fn remap32(value: f32, low1: f32, high1: f32, low2: f32, high2: f32) -> f32 {
    low2 + (value - low1) * (high2 - low2) / (high1 - low1)
}

pub fn make_my_local_sun_with_sky() -> EntityId {
    use ambient_api::core::{
        app::components::main_scene,
        rendering::components::{fog_height_falloff, light_diffuse, sky, sun},
        transform::components::rotation,
    };

    Entity::new().with(sky(), ()).spawn();

    Entity::new()
        .with(sun(), 0.0)
        .with(rotation(), Default::default())
        .with(main_scene(), ())
        .with(light_diffuse(), Vec3::ONE)
        // .with(fog_color(), vec3(0.88, 0.37, 0.34)) // dusty red
        .with(fog_color(), vec3(0.34, 0.37, 0.88)) // blueish. cold.
        // .with(fog_color(), vec3(0., 0., 0.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .spawn()
}
