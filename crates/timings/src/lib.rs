use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use ambient_ecs::{components, Debuggable, FrameEvent, Resource, System, World, WorldContext};
use ambient_sys::time::Instant;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use winit::event::{DeviceEvent, Event, WindowEvent};

/// Frame events being timed (in order!)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum TimingEventType {
    Input,
    AppSystemsStarted,
    ClientSystemsStarted,
    ClientSystemsFinished,
    DrawingWorld,
    DrawingUI,
    SubmittingGPUCommands,
    AppSystemsFinished,
    RenderingFinished,
}
impl TimingEventType {
    const COUNT: usize = Self::last().idx() + 1;

    const fn last() -> Self {
        Self::RenderingFinished
    }

    const fn idx(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TimingEvent {
    event_type: TimingEventType,
    time: Instant,
}
impl From<TimingEventType> for TimingEvent {
    fn from(event_type: TimingEventType) -> Self {
        Self {
            event_type,
            time: Instant::now(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FrameTimings {
    event_times: [Option<Instant>; TimingEventType::COUNT],
}
impl FrameTimings {
    pub fn input_to_rendered(&self) -> Option<Duration> {
        self.event_times[TimingEventType::Input as usize]
            .zip(self.event_times[TimingEventType::RenderingFinished as usize])
            .map(|(input, rendered)| rendered - input)
    }
}
impl std::fmt::Display for FrameTimings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut last = None;
        for (time, idx) in self
            .event_times
            .iter()
            .enumerate()
            .filter_map(|(idx, t)| t.zip(Some(idx)))
        {
            let event_type = TimingEventType::from_usize(idx).unwrap();
            if let Some(last) = last {
                write!(f, " <- {:?} -> ", time - last)?;
            }
            write!(f, "{:?}", event_type)?;
            last = Some(time);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    last_event_type: TimingEventType,
    event_times: [Option<Instant>; TimingEventType::COUNT],
}
impl Frame {
    fn timings(&self) -> FrameTimings {
        FrameTimings {
            event_times: self.event_times,
        }
    }

    fn should_accept_event(&self, event_type: TimingEventType) -> bool {
        // if it simply is the next event
        self.last_event_type.idx() + 1 == event_type.idx() ||
        // or if it is repeated input event (there can be multiple input events)
         (self.last_event_type == TimingEventType::Input && event_type == TimingEventType::Input) ||
        // or we don't have app systems finished event when rendering is finished (we don't have the callback in the browser)
         (self.last_event_type == TimingEventType::SubmittingGPUCommands && event_type == TimingEventType::RenderingFinished)
    }

    fn is_accepting_input(&self) -> bool {
        self.last_event_type == TimingEventType::Input
    }

    fn is_finished(&self) -> bool {
        self.last_event_type == TimingEventType::last()
    }

    fn process_event(&mut self, event: TimingEvent) {
        if self.should_accept_event(event.event_type) {
            self.event_times[event.event_type.idx()] =
                self.event_times[event.event_type.idx()].or(Some(event.time));
            self.last_event_type = event.event_type;
        }
    }
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            last_event_type: TimingEventType::Input,
            event_times: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Timings {
    ref_time: Instant,
    frames: Mutex<VecDeque<Frame>>,
}
impl Timings {
    fn report_timings(&self, timings: FrameTimings) {
        tracing::error!("TIMINGS: {}", timings);
        if let Some(input_to_rendered) = timings.input_to_rendered() {
            tracing::error!("INPUT TO RENDERED: {:?}", input_to_rendered);
        }
    }

    pub fn report_event(&self, event: impl Into<TimingEvent>) {
        let event = event.into();

        tracing::warn!(
            "TIMING EVENT: {:?} {:?}",
            event.event_type,
            event.time.duration_since(self.ref_time)
        );

        let mut frames = self.frames.lock().unwrap();
        for f in frames.iter_mut() {
            f.process_event(event);
        }

        if !frames.front().unwrap().is_accepting_input() {
            frames.push_front(Default::default());
        }
        while let Some(f) = frames.back() {
            if f.is_finished() {
                let timings = frames.pop_back().unwrap().timings();
                self.report_timings(timings);
            } else {
                break;
            }
        }
    }
}
impl Default for Timings {
    fn default() -> Self {
        // we normally have at most 2 frames in flight
        let mut frames = VecDeque::with_capacity(2);
        frames.push_back(Default::default());
        Self {
            ref_time: Instant::now(),
            frames: Mutex::new(frames),
        }
    }
}

components!("input", {
    @[Debuggable, Resource]
    timings: Arc<Timings>,
});

#[derive(Debug)]
struct TimingSystem<const APP_WORLD_EVENT_TYPE: usize, const CLIENT_WORLD_EVENT_TYPE: usize>;
impl<const APP_WORLD_EVENT_TYPE: usize, const CLIENT_WORLD_EVENT_TYPE: usize> System
    for TimingSystem<APP_WORLD_EVENT_TYPE, CLIENT_WORLD_EVENT_TYPE>
{
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        let time = Instant::now();
        let event_type = match world.context() {
            WorldContext::App => TimingEventType::from_usize(APP_WORLD_EVENT_TYPE).unwrap(),
            WorldContext::Client => TimingEventType::from_usize(CLIENT_WORLD_EVENT_TYPE).unwrap(),
            _ => return,
        };
        world
            .resource(timings())
            .report_event(TimingEvent { time, event_type });
    }
}

pub const fn on_started_timing_system() -> impl System {
    const APP_WORLD_EVENT_TYPE: usize = TimingEventType::AppSystemsStarted as usize;
    const CLIENT_WORLD_EVENT_TYPE: usize = TimingEventType::ClientSystemsStarted as usize;
    TimingSystem::<APP_WORLD_EVENT_TYPE, CLIENT_WORLD_EVENT_TYPE> {}
}

pub const fn on_finished_timing_system() -> impl System {
    const APP_WORLD_EVENT_TYPE: usize = TimingEventType::AppSystemsFinished as usize;
    const CLIENT_WORLD_EVENT_TYPE: usize = TimingEventType::ClientSystemsFinished as usize;
    TimingSystem::<APP_WORLD_EVENT_TYPE, CLIENT_WORLD_EVENT_TYPE> {}
}

#[derive(Debug)]
pub struct InputTimingSystem;
impl System<Event<'static, ()>> for InputTimingSystem {
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        if is_user_input_event(event) {
            world
                .resource_mut(timings())
                .report_event(TimingEventType::Input);
        }
    }
}

fn is_user_input_event(event: &Event<'static, ()>) -> bool {
    matches!(
        event,
        Event::WindowEvent {
            event: WindowEvent::ModifiersChanged(_)
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. },
            ..
        } | Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { .. },
            ..
        }
    )
}
