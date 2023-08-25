use std::{
    f32::consts::{PI, TAU},
    sync::Arc,
};

use ambient_app::{App, AppBuilder};
use ambient_audio::{track::Track, Attenuation, AudioEmitter, AudioListener, AudioStream, Source};
use ambient_core::{
    asset_cache,
    camera::{active_camera, far, near},
    main_scene,
    transform::{scale, translation},
};
use ambient_element::ElementComponentExt;
use ambient_native_std::math::SphericalCoords;
use ambient_primitives::Cube;
use ambient_renderer::{cast_shadows, color};
use ambient_ui_native::World;
use ambient_world_audio::{
    audio_emitter, audio_listener, audio_mixer, play_sound_on_entity, systems::setup_audio,
};
use glam::{vec3, vec4, Mat4, Vec3};
use parking_lot::Mutex;

fn spawn_emitters(world: &mut World) {
    let track = Track::from_vorbis(
        std::fs::read(
            "guest/rust/examples/games/pong/assets/Kevin_MacLeod_8bit_Dungeon_Boss_ncs.ogg",
        )
        .expect("Failed to read audio file")
        .to_vec(),
    )
    .unwrap();

    const COUNT: i32 = 1;
    const RADIUS: f32 = 8.0;
    for i in 0..COUNT {
        let theta = (i as f32) / COUNT as f32 * TAU;
        let pos = vec3(theta.cos() * RADIUS, theta.sin() * RADIUS, 2.0);

        let emitter = Arc::new(Mutex::new(AudioEmitter {
            amplitude: 50.0,
            attenuation: Attenuation::InversePoly {
                quad: 1.0,
                lin: 0.0,
                constant: 1.0,
            },
            pos,
        }));

        let id = Cube
            .el()
            .with(color(), vec4(0.7, 0.0, 0.7, 1.))
            .with(translation(), pos)
            .with(scale(), Vec3::splat(0.5))
            .with(cast_shadows(), ())
            .with(audio_emitter(), emitter)
            .spawn_static(world);

        play_sound_on_entity(world, id, track.decode().repeat()).expect("Failed to play sound");
    }
}

fn init(app: &mut App) {
    app.systems
        .add(Box::new(ambient_world_audio::systems::audio_systems()));

    let world = &mut app.world;
    let _assets = world.resource(asset_cache()).clone();

    // Floor
    let size = 16.0;
    Cube.el()
        .with(scale(), vec3(size, size, 1.))
        .with(cast_shadows(), ())
        .spawn_static(world);

    ambient_cameras::spherical::new(
        vec3(0., 0., 1.),
        SphericalCoords::new(1.5, 2.0 * PI * 4.0 / 8.0, 5.0),
    )
    .with(active_camera(), 0.)
    .with(
        audio_listener(),
        Arc::new(Mutex::new(AudioListener::new(
            Mat4::IDENTITY,
            Vec3::X * 0.3,
        ))),
    )
    .with(main_scene(), ())
    .with(near(), 1.)
    .with(far(), 8000.)
    .spawn(world);

    spawn_emitters(world);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    ambient_world_audio::init_components();

    let stream = AudioStream::new().unwrap();

    AppBuilder::simple()
        .run(|app, _| {
            setup_audio(&mut app.world).unwrap();
            app.world
                .add_resource(audio_mixer(), stream.mixer().clone());
            init(app)
        })
        .await;
}
