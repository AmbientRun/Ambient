use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct FpsCounter {
    start_time: Instant,
    current_frame_start: Instant,
    n_frames: u32,
    slowest_frame: Duration,
    active_time: Duration,
}
impl FpsCounter {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            current_frame_start: Instant::now(),
            n_frames: 0,
            slowest_frame: Duration::ZERO,
            active_time: Duration::ZERO,
        }
    }
    pub fn frame_start(&mut self) {
        self.current_frame_start = Instant::now();
    }
    pub fn frame_end(&mut self) -> Option<FpsSample> {
        let duration = self.start_time.elapsed();
        self.n_frames += 1;
        let frame_duration = self.current_frame_start.elapsed();
        self.slowest_frame = self.slowest_frame.max(frame_duration);
        self.active_time += frame_duration;
        if self.n_frames > 100 || duration.as_secs_f32() > 1. {
            let res =
                Some(FpsSample { n_frames: self.n_frames, duration, slowest_frame: self.slowest_frame, active_time: self.active_time });
            self.start_time = Instant::now();
            self.n_frames = 0;
            self.slowest_frame = Duration::ZERO;
            self.active_time = Duration::ZERO;
            return res;
        }
        None
    }
    pub fn frame_next(&mut self) -> Option<FpsSample> {
        let sample = self.frame_end();
        self.frame_start();
        sample
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FpsSample {
    pub n_frames: u32,
    pub duration: Duration,
    pub slowest_frame: Duration,
    pub active_time: Duration,
}
impl FpsSample {
    pub fn fps(&self) -> f32 {
        (self.n_frames as f32) / self.duration.as_secs_f32()
    }
    pub fn frame_time_ms(&self) -> f32 {
        self.duration.as_millis() as f32 / (self.n_frames as f32)
    }
    pub fn active_frame_time_ms(&self) -> f32 {
        self.active_time.as_millis() as f32 / (self.n_frames as f32)
    }
    pub fn activity_perc(&self, expected_frame_time_ms: f32) -> f32 {
        100. * self.active_frame_time_ms() / expected_frame_time_ms
    }
    pub fn dump_both(&self) -> String {
        format!("{:.2} fps, {:.2} ms/f", self.fps(), self.frame_time_ms())
    }
    pub fn dump_short(&self) -> String {
        let fps = self.fps();
        let fps = if fps > 1. { format!("{:.0}", fps) } else { format!("{:.2}", fps) };
        format!("{fps} fps/{:.1} ms max", self.slowest_frame.as_secs_f64() * 1000.)
    }
    pub fn dump_server(&self) -> String {
        format!("{:.1}%/{:.1} ms max", self.activity_perc(1000. / 60.), self.slowest_frame.as_secs_f64() * 1000.)
    }
}
