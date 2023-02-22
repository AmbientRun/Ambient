use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use once_cell::sync::Lazy;

use crate::{global::EventResult, internal::component::Entity};

type EventFuture = Pin<Box<dyn Future<Output = EventResult>>>;
type EventCallbackFn = Box<dyn Fn(&Entity) -> EventFuture>;
type EventCallbackFnOnce = Box<dyn FnOnce(&Entity) -> EventFuture>;

// the function is too general to be passed in directly
#[allow(clippy::redundant_closure)]
pub(crate) static EXECUTOR: Lazy<Executor> = Lazy::new(|| Executor::new());
static RAW_WAKER: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(std::ptr::null(), &RAW_WAKER),
    |_| {},
    |_| {},
    |_| {},
);

pub(crate) struct Executor {
    waker: Waker,
    current: RefCell<Vec<EventFuture>>,
    incoming: RefCell<Vec<Pin<Box<dyn Future<Output = EventResult>>>>>,
    current_callbacks: RefCell<Callbacks>,
    incoming_callbacks: RefCell<Callbacks>,
    frame_state: RefCell<FrameState>,
}
// WebAssembly, at time of writing, is single-threaded. This is a convenient little lie
// to make it easy to use this in a global context.
unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

impl Executor {
    pub fn new() -> Self {
        Executor {
            waker: unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &RAW_WAKER)) },
            current: RefCell::new(Default::default()),
            incoming: RefCell::new(Default::default()),
            current_callbacks: RefCell::new(Default::default()),
            incoming_callbacks: RefCell::new(Default::default()),
            frame_state: RefCell::new(Default::default()),
        }
    }

    pub fn execute(&self, frame_state: FrameState, event_name: &str, components: &Entity) {
        *self.frame_state.borrow_mut() = frame_state;

        // Load all pending callbacks.
        {
            let mut incoming = self.incoming_callbacks.borrow_mut();
            let mut current = self.current_callbacks.borrow_mut();

            for (event_name, mut new_callbacks) in incoming.on.drain() {
                current
                    .on
                    .entry(event_name)
                    .or_default()
                    .append(&mut new_callbacks);
            }
            for (event_name, mut new_callbacks) in incoming.once.drain() {
                current
                    .once
                    .entry(event_name)
                    .or_default()
                    .append(&mut new_callbacks);
            }
        }

        // Dispatch all callbacks.
        {
            let mut new_futures = vec![];
            let mut callbacks = self.current_callbacks.borrow_mut();
            if let Some(callbacks) = callbacks.on.get(event_name) {
                for callback in callbacks {
                    new_futures.push(callback(components));
                }
            }

            for callback in callbacks.once.remove(event_name).unwrap_or_default() {
                new_futures.push(callback(components));
            }

            // This must be done as a separate step as `callback` could mutate `self.incoming`.
            self.incoming.borrow_mut().append(&mut new_futures);
        }

        // Load all pending futures into current.
        {
            let (mut current, mut incoming) =
                (self.current.borrow_mut(), self.incoming.borrow_mut());
            current.append(&mut incoming);
        }

        // Run all current futures.
        // These are extracted to ensure that a panic will not result in the same
        // tasks being executed forever.
        {
            let mut futures = std::mem::take(&mut *self.current.borrow_mut());
            futures.retain_mut(
                |f| match f.as_mut().poll(&mut Context::from_waker(&self.waker)) {
                    Poll::Ready(Ok(_)) => false,
                    Poll::Ready(Err(e)) => {
                        eprintln!("Error while handling future: {e:?}");
                        false
                    }
                    Poll::Pending => true,
                },
            );
            *self.current.borrow_mut() = futures;
        }
    }

    pub fn frame_state(&self) -> Ref<'_, FrameState> {
        self.frame_state.borrow()
    }

    pub fn register_callback(&self, event_name: String, callback: EventCallbackFn) {
        self.incoming_callbacks
            .borrow_mut()
            .on
            .entry(event_name)
            .or_default()
            .push(callback);
    }

    pub fn register_callback_once(&self, event_name: String, callback: EventCallbackFnOnce) {
        self.incoming_callbacks
            .borrow_mut()
            .once
            .entry(event_name)
            .or_default()
            .push(callback);
    }

    pub fn spawn(&self, fut: EventFuture) {
        self.incoming.borrow_mut().push(fut);
    }
}

#[derive(Default)]
pub struct FrameState {
    time: f32,
}
impl FrameState {
    pub fn new(time: f32) -> Self {
        Self { time }
    }

    pub fn time(&self) -> f32 {
        self.time
    }
}

#[derive(Default)]
struct Callbacks {
    on: HashMap<String, Vec<EventCallbackFn>>,
    once: HashMap<String, Vec<EventCallbackFnOnce>>,
}
