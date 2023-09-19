use std::f32::consts::{E, PI};

use ambient_api::{
    core::{
        audio::components::amplitude,
        camera::components::{fog, perspective_infinite_reverse},
        player::components::is_player,
        primitives::concepts::Sphere,
        rendering::components::{
            cast_shadows, color, fog_color, fog_density, light_ambient, outline,
            pbr_material_from_url, transparency_group,
        },
        transform::components::{local_to_world, scale, spherical_billboard, translation},
    },
    prelude::*,
};
use packages::{
    character_animation::{self, components::basic_character_animations},
    temperature::components::{temperature, temperature_src_radius, temperature_src_rate},
    this::components::{ambient_loop, heat_offset, heat_radius, visualizing_heat_source},
};

const DEATH_TEMP: f32 = 21.;
const NORMAL_TEMP: f32 = 37.;

#[main]
pub fn main() {
    visualize_sources_of_warmth();

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
            entity::get_component(player::get_local(), temperature()).unwrap_or(NORMAL_TEMP),
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
                *foggy = *foggy * 0.9 + 0.1 * (0.01 + 0.18 * t);
            });
        } else {
            let t = (coldness - 0.60) / (1. - 0.60);
            entity::set_component(sun, fog_density(), 0.20 + 0.80 * t * t);
        }
        // let desired_fog_colour =
        //     vec3(0.75, 0.45, 0.75).lerp(vec3(0.60, 1.00, 1.00), coldness.sqrt());
        // entity::mutate_component(sun, fog_color(), |color| {
        //     *color = color.lerp(desired_fog_colour, 0.1)
        // });
    });

    spawn_query(())
        .requires((is_player(), basic_character_animations()))
        .bind(|plrs| {
            for (plr, _) in plrs {
                entity::add_components(
                    plr,
                    Entity::new()
                        .with(
                            character_animation::components::idle(),
                            anim_url("movement/offensive idle"),
                        )
                        .with(
                            character_animation::components::walk_forward(),
                            anim_url("movement/Jog Forward"),
                        )
                        .with(
                            character_animation::components::walk_forward_left(),
                            anim_url("movement/Jog Forward Diagonal"),
                        )
                        .with(
                            character_animation::components::walk_forward_right(),
                            anim_url("movement/Jog Forward Diagonal (1)"),
                        )
                        .with(
                            character_animation::components::walk_right(),
                            anim_url("movement/Jog Strafe Right"),
                        )
                        .with(
                            character_animation::components::walk_backward(),
                            anim_url("movement/Jog Forward"),
                        )
                        .with(
                            character_animation::components::walk_backward_left(),
                            anim_url("movement/Jog Backward Diagonal"),
                        )
                        .with(
                            character_animation::components::walk_backward_right(),
                            anim_url("movement/Jog Backward Diagonal (1)"),
                        )
                        .with(
                            character_animation::components::walk_left(),
                            anim_url("movement/Jog Strafe Left"),
                        ),
                );
            }
        });
}

fn anim_url(name: &str) -> String {
    packages::this::assets::url(&format!("{name}.fbx/animations/mixamo.com.anim"))
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
        // .with(rotation(), Default::default())
        .with(main_scene(), ())
        // .with(light_diffuse(), Vec3::ONE) // pure white light
        // .with(light_ambient(), vec3(0.100, 0.100, 0.100)) // low ambience
        .with(light_diffuse(), Vec3::splat(0.90)) // hi diffuse
        .with(light_ambient(), Vec3::splat(0.15)) // low ambience
        // .with(fog_color(), vec3(0.88, 0.37, 0.34)) // dusty red
        // .with(fog_color(), vec3(0.34, 0.37, 0.88)) // blueish. cold.
        // .with(fog_color(), vec3(0.804, 0.804, 0.804)) // grey of the website
        .with(fog_color(), vec3(0.850, 0.850, 0.850))
        // .with(fog_color(), vec3(0., 0., 0.))
        .with(fog_density(), 0.1)
        .with(fog_height_falloff(), 0.01)
        // .with(rotation(), Quat::from_rotation_y(190.0f32.to_radians()))
        .with(
            rotation(),
            Quat::from_xyzw(-0.091639765, 0.9358677, -0.312692, 0.13407977)
                * Quat::from_rotation_z(PI),
        )
        .spawn()
}

const HEAT_SOURCE_COLOUR: Vec3 = Vec3::new(1.00, 0.00, 0.00);
const ORANGERIET_2: Vec3 = Vec3::new(0.9882352941176471, 0.4196078431372549, 0.3137254901960784);
const ORANGERIET_2_BRIGHT: Vec3 = Vec3::new(0.99, 0.76, 0.65);

fn visualize_sources_of_warmth() {
    spawn_query((
        translation(),
        temperature_src_rate(),
        temperature_src_radius(),
    ))
    .bind(|heat_sources| {
        for (heat_source, (pos, heat, radius)) in heat_sources {
            if heat > 2.2 {
                // campfire
                spawn_heat_visualizing_sphere(heat_source, radius * 0.30, 0.);
                spawn_heat_visualizing_sphere(heat_source, radius * 0.34, 0.);
                spawn_heat_visualizing_sphere(heat_source, radius * 0.38, 0.);
            } else if heat > 0.0 {
                // players beating heart - removed for now
                // spawn_heat_visualizing_sphere(heat_source, radius * 0.30, 1.25);
            }

            // do not create for negative heat sources
        }
    });
    let _player_entity = player::get_local();
    query((visualizing_heat_source(), heat_radius())).each_frame(move |spheres| {
        for (sphere, (hs, radius)) in spheres {
            if let Some(hs_translation) = entity::get_component(hs, translation()) {
                entity::add_component(
                    sphere,
                    translation(),
                    hs_translation
                        + entity::get_component(sphere, heat_offset()).unwrap_or_default(),
                );
                let phase_offset = radius * -0.12;
                entity::mutate_component(sphere, scale(), move |scale| {
                    *scale = scale.lerp(
                        Vec3::splat(
                            radius
                                * (1.00
                                    + 0.10
                                        * ((game_time().as_secs_f32() + phase_offset) * 2.0).sin()),
                        ),
                        0.2,
                    );
                    // scale.y = 0.01; // for billboard
                });
            }
        }
    });
}

fn spawn_heat_visualizing_sphere(
    heat_source: EntityId,
    sphere_radius: f32,
    height_offset: f32,
) -> EntityId {
    let sphere = Sphere {
        ..Sphere::suggested()
    }
    .make()
    .with(visualizing_heat_source(), heat_source)
    .with(transparency_group(), 5)
    .with(color(), HEAT_SOURCE_COLOUR.extend(0.1))
    .with(local_to_world(), Mat4::IDENTITY)
    .with(scale(), Vec3::ONE)
    // .with(spherical_billboard(), ())
    .with(heat_radius(), sphere_radius)
    .with(heat_offset(), vec3(0., 0., height_offset))
    .spawn();
    entity::add_child(heat_source, sphere);
    sphere
}
