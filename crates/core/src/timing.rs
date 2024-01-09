use std::sync::atomic::AtomicBool;

use ambient_ecs::{components, Debuggable, Resource};
use ambient_sys::time::Instant;

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Frame events being timed (in order!)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimingEventType {
    Input,
    ScriptingStarted,
    ScriptingFinished,
    ClientSystemsStarted,
    ClientSystemsFinished,
    DrawingWorld,
    DrawingUI,
    SubmittingGPUCommands,
    RenderingFinished,
}
impl TimingEventType {
    pub const COUNT: usize = Self::last().idx() + 1;

    pub const fn last() -> Self {
        Self::RenderingFinished
    }

    pub const fn idx(self) -> usize {
        self as usize
    }

    pub const fn from_usize(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Input),
            1 => Some(Self::ScriptingStarted),
            2 => Some(Self::ScriptingFinished),
            3 => Some(Self::ClientSystemsStarted),
            4 => Some(Self::ClientSystemsFinished),
            5 => Some(Self::DrawingWorld),
            6 => Some(Self::DrawingUI),
            7 => Some(Self::SubmittingGPUCommands),
            8 => Some(Self::RenderingFinished),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TimingEvent {
    pub event_type: TimingEventType,
    pub time: Instant,
}
impl From<TimingEventType> for TimingEvent {
    fn from(event_type: TimingEventType) -> Self {
        Self {
            event_type,
            time: Instant::now(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reporter {
    sender: flume::Sender<TimingEvent>,
    receiver: flume::Receiver<TimingEvent>,
}
impl Default for Reporter {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();
        Self { sender, receiver }
    }
}
impl Reporter {
    pub fn report_event(&self, event: impl Into<TimingEvent>) {
        if ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            self.sender.send(event.into()).ok();
        }
    }

    pub fn reporter(&self) -> ThinReporter {
        if ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            ThinReporter::Enabled(self.sender.clone())
        } else {
            ThinReporter::Disabled
        }
    }

    pub fn try_iter(&self) -> impl Iterator<Item = TimingEvent> + '_ {
        self.receiver.try_iter()
    }
}

pub enum ThinReporter {
    Disabled,
    Enabled(flume::Sender<TimingEvent>),
}
impl ThinReporter {
    pub fn report_event(&self, event: impl Into<TimingEvent>) {
        match self {
            Self::Disabled => {}
            Self::Enabled(sender) => {
                sender.send(event.into()).ok();
            }
        }
    }
}

components!("timing", {
    @[Debuggable, Resource]
    reporter: Reporter,
});
