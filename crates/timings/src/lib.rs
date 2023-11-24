use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use ambient_ecs::{components, Debuggable, FrameEvent, Resource, System, World};
use ambient_sys::time::Instant;
use winit::event::{DeviceEvent, Event, WindowEvent};

#[derive(Default, Debug)]
pub struct Timings {
    // pub earliest_input_time: Mutex<Option<Instant>>, // FIXME
}
impl Timings {
    pub fn report_input(&self) {
        tracing::error!("{:?} reported INPUT", self as *const Self);
        // let mut ts = self.earliest_input_time.lock().unwrap();
        // if ts.is_none() {
        let now = Instant::now();
        tracing::error!("{:?} INPUT EVENT: {:?}", self as *const Self, now);
        // *ts = Some(now);
        // }
    }

    pub fn report_gpu_commands_submitted(&self) {
        // let latency = self.earliest_input_time.lock().unwrap().take().map(|t| t.elapsed());
        tracing::error!(
            "{:?} GPU COMMANDS EVENT: {:?}",
            self as *const Self,
            Instant::now()
        );
    }

    pub fn report_rendering_finished(&self) {
        tracing::error!(
            "{:?} GPU rendering EVENT: {:?}",
            self as *const Self,
            Instant::now()
        );
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
        tracing::error!("FRAME EVENT {:?} {}", world as *const World, world.name());
        // let latency = world.resource_mut(timings()).clear();
        // if let Some(latency) = latency {
        //     // FIXME
        //     tracing::error!("Latency: {:?}", latency);
        // }
    }
}

impl System<Event<'static, ()>> for TimingSystem {
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        if is_user_input_event(event) {
            tracing::error!("USER_INPUT EVENT {:?}", world as *const World);
            world.resource_mut(timings()).report_input();
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
