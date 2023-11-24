use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use ambient_ecs::{components, Debuggable, FrameEvent, Resource, System, World, WorldContext};
use ambient_sys::time::Instant;
use winit::event::{DeviceEvent, Event, WindowEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimingEventType {
    Input,
    SystemsStarted,
    // RenderingStarted,
    // GpuCommandsSubmitted,
    RenderingFinished,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FrameState {
    ReceivingInput,
    RunningSystems,
    // Rendering,
    // SubmittingGpuCommands,
    // WaitingForGpu,
}

#[derive(Clone, Copy, Debug)]
pub struct FrameTimings {
    pub input_to_rendered: Option<Duration>,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    state: FrameState,
    first_input_time: Option<Instant>,
}
impl Frame {
    fn process_event(&mut self, event: TimingEvent) -> Option<FrameTimings> {
        match (self.state, event.event_type) {
            (FrameState::ReceivingInput, TimingEventType::Input) => {
                self.first_input_time = self.first_input_time.or(Some(event.time));
                None
            }
            (FrameState::ReceivingInput, TimingEventType::SystemsStarted) => {
                self.state = FrameState::RunningSystems;
                None
            }
            (FrameState::RunningSystems, TimingEventType::RenderingFinished) => {
                let input_to_rendered = self.first_input_time.map(|t| event.time - t);
                Some(FrameTimings { input_to_rendered })
            }
            _ => None,
        }
    }
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            state: FrameState::ReceivingInput,
            first_input_time: None,
        }
    }
}

#[derive(Debug)]
pub struct Timings {
    ref_time: Instant,
    // pub earliest_input_time: Mutex<Option<Instant>>, // FIXME
    current_frame: Mutex<Frame>,
    pending_frames: Mutex<Vec<Frame>>,
}
impl Timings {
    fn report_timings(&self, timings: FrameTimings) {
        if let Some(input_to_rendered) = timings.input_to_rendered {
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

        let mut frames = self.pending_frames.lock().unwrap();
        frames.retain_mut(|f| {
            if let Some(ft) = f.process_event(event) {
                self.report_timings(ft);
                false
            } else {
                true
            }
        });

        let mut frame = self.current_frame.lock().unwrap();
        let ft = frame.process_event(event);
        if frame.state != FrameState::ReceivingInput {
            frames.push(*frame);
            *frame = Default::default();
        }
        if let Some(ft) = ft {
            self.report_timings(ft);
        }
    }
}
impl Default for Timings {
    fn default() -> Self {
        Self {
            ref_time: Instant::now(),
            current_frame: Default::default(),
            pending_frames: Default::default(),
        }
    }
}

components!("input", {
    @[Debuggable, Resource]
    timings: Arc<Timings>,
});

#[derive(Debug)]
pub struct TimingSystem;

impl System for TimingSystem {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        if world.context() == WorldContext::Client {
            world
                .resource(timings())
                .report_event(TimingEventType::SystemsStarted);
        }
        // match world.name() {
        //     "main_app" => {
        //         tracing::error!("running main app systems FRAME EVENT {:?} {} {:?}", world as *const World, world.name(), world.context());
        //         // input gets processed here - no more input gets captured for this frame at this point
        //     }
        //     "client_game_world" => {
        //         tracing::error!("running systems - FRAME EVENT {:?} {} {:?}", world as *const World, world.name(), world.context());
        //     }
        //     _ => {
        //     }
        // }
    }
}

impl System<Event<'static, ()>> for TimingSystem {
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
