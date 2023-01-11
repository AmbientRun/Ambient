use std::sync::Arc;

use anyhow::Context;
use derive_more::{Deref, DerefMut, From, Into};
use elements_audio::{hrtf::HrtfLib, Attenuation, AudioEmitter, AudioListener, AudioMixer, Sound, Source};
use elements_ecs::{components, query, EntityId, World};
use elements_element::ElementComponentExt;
use elements_std::Cb;
use elements_ui::{
    graph::{Graph, GraphStyle}, Editor, FlowColumn
};
use glam::{vec2, vec4};
use itertools::Itertools;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

components!("audio", {
    hrtf_lib: Arc<HrtfLib>,
    audio_emitter: Arc<Mutex<AudioEmitter>>,
    audio_listener: Arc<Mutex<AudioListener>>,

    audio_mixer: AudioMixer,
});

/// TODO: hook this into the Attenuation inside elements_audio
#[derive(Serialize, Deserialize, Debug, Clone, Copy, DerefMut, Deref, From, Into)]
pub struct AttenuationEditorVisual(Attenuation);

impl Editor for AttenuationEditorVisual {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, opts: elements_ui::EditorOpts) -> elements_element::Element {
        let editor = Attenuation::editor(
            *value,
            Some(Cb::new(move |v| {
                if let Some(on_change) = on_change.as_ref() {
                    on_change(v.into())
                }
            })),
            opts,
        );

        let x_max = value.inverse(0.01);

        const STEPS: u32 = 32;

        let points = (0..STEPS)
            .map(|v| {
                let x = (v as f32 / (STEPS - 1) as f32) * x_max;

                let y = value.attenuate(x).clamp(0.0, 2.0);

                vec2(x, y)
            })
            .collect_vec();

        let graph = Graph {
            points,
            style: GraphStyle { color: vec4(0.0, 0.0, 1.0, 1.0), ..Default::default() },
            width: 400.0,
            height: 200.0,
            ..Default::default()
        }
        .el();

        FlowColumn::el([editor, graph])
    }
}

fn get_audio_listener(world: &World) -> anyhow::Result<&Arc<Mutex<AudioListener>>> {
    let (_, listener) = query(audio_listener())
        .iter(world, None)
        .exactly_one()
        .map_err(|v| anyhow::anyhow!("Incorrect number of listeners in world. Additional: {:?}", v.count()))?;

    Ok(listener)
}

/// Makes a sound source emit from the entity
pub fn play_sound_on_entity<S: 'static + Source>(world: &World, id: EntityId, source: S) -> anyhow::Result<Sound> {
    let hrtf_lib = world.resource(hrtf_lib());
    let mixer = world.resource(audio_mixer());
    let emitter = world.get_ref(id, audio_emitter()).context("No audio emitter on entity")?;

    let listener = get_audio_listener(world)?;

    Ok(mixer.play(source.spatial(hrtf_lib, listener.clone(), emitter.clone())))
}
