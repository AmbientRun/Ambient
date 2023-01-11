use std::{io::Cursor, sync::Arc};

use elements_audio::{hrtf::HrtfLib, AudioMixer};
use elements_core::transform::local_to_world;
use elements_ecs::{query, SystemGroup, World};
use glam::{vec4, Mat4};

use crate::{audio_emitter, audio_listener, audio_mixer, hrtf_lib};

/// Initializes the HRTF sphere and adds the appropriate resources
///
/// TODO: customizer IR sphere selection
pub fn setup_audio(world: &mut World, mixer: AudioMixer) -> anyhow::Result<()> {
    let hrtf = Arc::new(HrtfLib::load(Cursor::new(include_bytes!("../IRC_1002_C.bin")))?);
    world.add_resource(hrtf_lib(), hrtf);

    world.add_resource(audio_mixer(), mixer);

    Ok(())
}

/// This translates elements RHS Z-up coordinate system to the HRIR sphere LHS Y-up
/// https://github.com/mrDIMAS/hrir_sphere_builder/blob/e52a10ece678a2b80a0978f7cf23f3ad9cce41c3/src/hrtf_builder.cpp#L155-L162
pub const Y_UP_LHS: Mat4 =
    Mat4::from_cols(vec4(1.0, 0.0, 0.0, 0.0), vec4(0.0, 0.0, 1.0, 0.0), vec4(0.0, 1.0, 0.0, 0.0), vec4(0.0, 0.0, 0.0, 1.0));

pub fn spatial_audio_systems() -> SystemGroup {
    SystemGroup::new(
        "spatial_audio",
        vec![
            // Updates the volume of audio emitters in the world
            query((audio_emitter(), local_to_world())).to_system(|q, world, qs, _| {
                for (_, (emitter, ltw)) in q.iter(world, qs) {
                    let (_, _, pos) = ltw.to_scale_rotation_translation();
                    let mut emitter = emitter.lock();
                    emitter.pos = pos;
                }
            }),
            query((audio_listener(), local_to_world())).to_system_with_name("update_audio_listener", |q, world, qs, _| {
                for (_, (listener, &ltw)) in q.iter(world, qs) {
                    let mut listener = listener.lock();
                    listener.transform = Y_UP_LHS * ltw;
                }
            }),
        ],
    )
}

pub fn client_systems() -> SystemGroup {
    SystemGroup::new("Spatial audio", vec![Box::new(spatial_audio_systems())])
}
