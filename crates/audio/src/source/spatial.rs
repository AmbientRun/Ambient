use std::{ops::Deref, time::Duration};

use glam::Vec3;

use crate::{
    hrtf::{Hrtf, HrtfContext, HrtfLib},
    value::Value,
    AudioEmitter, AudioListener, Frame, Source, MAX_ANGULAR_SPEED, MAX_SPEED,
};

#[derive(Debug)]
pub struct Spatial<S, L, E> {
    hrtf: Hrtf<S>,
    /// Keep track of the previous position to not move the source too fast from one block to the
    /// next.
    prev_to_source: Vec3,
    output_buffer: Box<[Frame]>,
    len: usize,
    cur: usize,
    listener: L,
    emitter: E,
}

const BLOCK_DURATION: Duration = Duration::from_millis(15);
const INTERPOLATION_STEPS: u32 = 8;

impl<S, L, E> Spatial<S, L, E>
where
    S: Source,
    L: for<'x> Value<'x, Item = AudioListener>,
    E: for<'x> Value<'x, Item = AudioEmitter>,
{
    pub fn new(source: S, hrtf_lib: &HrtfLib, listener: L, emitter: E) -> Self {
        let sample_rate = source.sample_rate();
        let block_len = (sample_rate as f32 * BLOCK_DURATION.as_secs_f32()).round() as _;
        let buf_len = block_len * INTERPOLATION_STEPS as usize;

        let ctx = {
            let listener = listener.get();
            let emitter = emitter.get();

            Self::calculate_hrtf_context(
                listener
                    .deref()
                    .transform()
                    .inverse()
                    .transform_point3(emitter.pos),
                &listener,
                &emitter,
            )
        };

        Self {
            hrtf: Hrtf::new(hrtf_lib, source, ctx, block_len, INTERPOLATION_STEPS),
            emitter,
            listener,
            output_buffer: vec![Frame::ZERO; buf_len].into_boxed_slice(),
            len: 0,
            cur: 0,
            prev_to_source: ctx.to_source(),
        }
    }

    fn calculate_hrtf_context(
        prev_to_source: Vec3,
        listener: &AudioListener,
        emitter: &AudioEmitter,
    ) -> HrtfContext {
        let listener_inv = listener.transform.inverse();

        // Limit the velocity of the source to avoid clipping
        let to_source = listener_inv.transform_point3(emitter.pos);
        let mut rel = to_source - prev_to_source;

        // Limit the angular velocity to avoid IR sphere clipping
        let radial = prev_to_source;

        // Perceived velocity perpendicular to the radial vector.
        let w = ang_vel(rel, radial);

        // Recalculate the velocity with a clipped angular velocity
        if w > MAX_ANGULAR_SPEED {
            rel *= MAX_ANGULAR_SPEED / w
        }

        let to_source = (rel).clamp_length_max(MAX_SPEED) + prev_to_source;

        HrtfContext::new(
            to_source,
            listener.ear_distance / 2.0,
            emitter.attenuation,
            emitter.amplitude,
        )
    }
}

fn ang_vel(v: Vec3, radial: Vec3) -> f32 {
    let rlen = radial.length();
    let perp = v.reject_from_normalized(radial / rlen);
    perp.length() / rlen
}

impl<S, L, E> Source for Spatial<S, L, E>
where
    S: Send + Source,
    L: for<'x> Value<'x, Item = AudioListener>,
    E: for<'x> Value<'x, Item = AudioEmitter>,
{
    #[inline(always)]
    fn next_sample(&mut self) -> Option<crate::Frame> {
        if self.cur < self.len {
            let s = self.output_buffer[self.cur];
            self.cur += 1;
            Some(s)
        } else {
            let ctx = {
                let listener = self.listener.get();
                let emitter = self.emitter.get();

                let ctx = Self::calculate_hrtf_context(self.prev_to_source, &listener, &emitter);
                self.prev_to_source = ctx.to_source();

                ctx
            };

            let new_len = self.hrtf.process(ctx, &mut self.output_buffer);
            self.len = new_len;
            self.cur = 1;
            if new_len == 0 {
                return None;
            }

            Some(self.output_buffer[0])
        }
    }

    fn sample_rate(&self) -> crate::SampleRate {
        self.hrtf.source().sample_rate()
    }

    fn sample_count(&self) -> Option<u64> {
        self.hrtf.source().sample_count()
    }
}
