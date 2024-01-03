use std::{collections::VecDeque, time::Duration};

use ambient_core::timing::{reporter, TimingEvent, TimingEventType};
use ambient_ecs::{components, Debuggable, FrameEvent, Resource, System, World, WorldContext};
use ambient_sys::time::Instant;
use winit::event::{DeviceEvent, Event, WindowEvent};

const MAX_SAMPLES: usize = 128;

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
pub struct ProcessTimingEventsSystem {
    frames: VecDeque<Frame>,
}
impl Default for ProcessTimingEventsSystem {
    fn default() -> Self {
        // we normally have at most 2 frames in flight
        let mut frames = VecDeque::with_capacity(2);
        frames.push_back(Default::default());
        Self { frames }
    }
}
impl System for ProcessTimingEventsSystem {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        // only process timing events in the app world so that they are available for the debugger
        if world.context() != WorldContext::App {
            return;
        }

        // timings of finished frames, to be added to the samples resource
        let mut pending_samples = Vec::new();

        let reporter = world.resource(reporter());
        for event in reporter.try_iter() {
            for f in self.frames.iter_mut() {
                f.process_event(event);
            }

            // the newest frame should always be accepting input
            if !self
                .frames
                .front()
                .map(Frame::is_accepting_input)
                .unwrap_or_default()
            {
                // the currently newest one is missing or not accepting input, so we need to add a new one
                self.frames.push_front(Default::default());
            }

            // pop finished frames
            while let Some(f) = self.frames.back() {
                if f.is_finished() {
                    let timings = self.frames.pop_back().unwrap().timings();
                    pending_samples.push(timings);
                } else {
                    break;
                }
            }
        }

        // add pending samples to the samples resource
        if !pending_samples.is_empty() {
            let samples = world.resource_mut(samples());
            for sample in pending_samples {
                while samples.len() + 1 >= MAX_SAMPLES {
                    samples.pop_front();
                }
                samples.push_back(sample);
            }
        }
    }
}

components!("timings", {
    @[Debuggable, Resource]
    samples: VecDeque<FrameTimings>,
});

#[derive(Debug)]
struct ClientWorldTimingSystem<const EVENT_TYPE: usize>;
impl<const EVENT_TYPE: usize> System for ClientWorldTimingSystem<EVENT_TYPE> {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        if world.context() != WorldContext::Client {
            return;
        }

        // emit timing events in the client world
        let time = Instant::now();
        let event_type = TimingEventType::from_usize(EVENT_TYPE).unwrap();
        world
            .resource(reporter())
            .report_event(TimingEvent { time, event_type });
    }
}

pub const fn on_started_timing_system() -> impl System {
    const EVENT_TYPE: usize = TimingEventType::ClientSystemsStarted as usize;
    ClientWorldTimingSystem::<EVENT_TYPE> {}
}

pub const fn on_finished_timing_system() -> impl System {
    const EVENT_TYPE: usize = TimingEventType::ClientSystemsFinished as usize;
    ClientWorldTimingSystem::<EVENT_TYPE> {}
}

#[derive(Debug)]
struct SystemWrapper<S> {
    system: S,
    on_started: TimingEventType,
    on_finished: TimingEventType,
}
impl<S, E> System<E> for SystemWrapper<S>
where
    S: System<E>,
{
    fn run(&mut self, world: &mut World, event: &E) {
        let r = world.resource(reporter()).reporter();
        r.report_event(TimingEvent::from(self.on_started));
        self.system.run(world, event);
        r.report_event(TimingEvent::from(self.on_finished));
    }
}

pub fn wrap_system<E>(
    system: impl System<E>,
    on_started: TimingEventType,
    on_finished: TimingEventType,
) -> impl System<E> {
    SystemWrapper {
        system,
        on_started,
        on_finished,
    }
}

#[derive(Debug)]
pub struct InputTimingSystem;
impl System<Event<'static, ()>> for InputTimingSystem {
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        if is_user_input_event(event) {
            world
                .resource(reporter())
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
