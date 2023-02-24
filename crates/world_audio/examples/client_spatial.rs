use std::{f32::consts::TAU, sync::Arc};

use ambient_app::{App, AppBuilder};
use ambient_audio::{track::Track, Attenuation, AudioEmitter, AudioListener, AudioStream, Source};
use ambient_core::{
    asset_cache,
    camera::{active_camera, far, near},
    main_scene,
    transform::{scale, translation},
};
use ambient_element::ElementComponentExt;
use ambient_primitives::Cube;
use ambient_renderer::{cast_shadows, color};
use ambient_std::math::SphericalCoords;
use ambient_ui::World;
use ambient_world_audio::{audio_emitter, audio_listener, play_sound_on_entity, systems::setup_audio};
use glam::{vec3, vec4, Mat4, Vec3};
use parking_lot::Mutex;

fn spawn_emitters(world: &mut World) {
    let track = Track::from_wav(std::fs::read("../../../elements/example_assets/ambience.wav").unwrap().to_vec()).unwrap();

    let count = 1;
    for i in 0..count {
        let theta = (i as f32) / count as f32 * TAU;
        let pos = vec3(theta.cos() * 16.0, theta.sin() * 16.0, 2.0);

        let emitter = Arc::new(Mutex::new(AudioEmitter {
            amplitude: 5.0,
            attenuation: Attenuation::InversePoly { quad: 0.1, lin: 0.0, constant: 1.0 },
            pos,
        }));

        let id = Cube
            .el()
            .set(color(), vec4(0.7, 0.0, 0.7, 1.))
            .set(translation(), pos)
            .set(scale(), Vec3::splat(0.5))
            .set(cast_shadows(), ())
            .set(audio_emitter(), emitter)
            .spawn_static(world);

        play_sound_on_entity(world, id, track.decode().repeat()).expect("Failed to play sound");
    }
}

fn init(app: &mut App) {
    app.systems.add(Box::new(ambient_world_audio::systems::spatial_audio_systems()));

    let world = &mut app.world;
    let _assets = world.resource(asset_cache()).clone();

    // Floor
    let size = 128.0;
    Cube.el().set(scale(), vec3(size, size, 1.)).set_default(cast_shadows()).spawn_static(world);

    ambient_cameras::spherical::new(vec3(0., 0., 0.), SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.))
        .set(active_camera(), 0.)
        .set(audio_listener(), Arc::new(Mutex::new(AudioListener::new(Mat4::IDENTITY, Vec3::X * 0.3))))
        .set(main_scene(), ())
        .set(near(), 1.)
        .set(far(), 8000.)
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
            setup_audio(&mut app.world, stream.mixer().clone()).unwrap();
            init(app)
        })
        .await;
}
