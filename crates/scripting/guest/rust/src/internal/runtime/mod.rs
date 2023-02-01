use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::Components;

mod guest_conversion;

wit_bindgen_guest_rust::export!("src/internal/guest.wit");

/// In Rust, functions that can fail are expected to return a `Result` type.
/// `EventResult` is a `Result` type that has no value and automatically
/// captures errors for you, which is why it's used as the return type
/// event handlers.
///
/// This accepts any kind of error,
/// so you can use the question-mark operator `?` to bubble errors up.
pub type EventResult = anyhow::Result<()>;

/// The default "happy path" value for an `EventResult`. You can return this
/// from an event handler to signal that everything's OK.
#[allow(non_upper_case_globals)]
pub const EventOk: EventResult = Ok(());

type EventCallbackFn = Box<dyn Fn(&Components) -> Pin<Box<dyn Future<Output = EventResult>>>>;
type EventCallbackFnOnce = Box<dyn FnOnce(&Components) -> Pin<Box<dyn Future<Output = EventResult>>>>;

type CallbackMap = HashMap<String, Vec<EventCallbackFn>>;
type OnceCallbackMap = HashMap<String, Vec<EventCallbackFnOnce>>;

struct SchedulerState {
    callbacks: CallbackMap,
    once_callbacks: OnceCallbackMap,
    futures: Vec<Pin<Box<dyn Future<Output = EventResult>>>>,
    waker: Waker,
}
impl SchedulerState {
    fn execute_futures(&mut self) {
        let mut futures = std::mem::take(&mut self.futures);
        futures.retain_mut(|f| match f.as_mut().poll(&mut Context::from_waker(&self.waker)) {
            Poll::Ready(Ok(_)) => false,
            Poll::Ready(Err(e)) => {
                eprintln!("Error while handling future: {e:?}");
                false
            }
            Poll::Pending => true,
        });
        self.futures = futures;
    }
}
struct PendingSchedulerState {
    callbacks: CallbackMap,
    once_callbacks: OnceCallbackMap,
    futures: Vec<Pin<Box<dyn Future<Output = EventResult>>>>,
}
struct FrameState {
    time: f32,
    frametime: f32,
}
impl FrameState {
    const fn new() -> Self {
        Self { time: 0.0, frametime: 0.0 }
    }
}
struct GlobalState {
    scheduler: RefCell<Option<SchedulerState>>,
    pending_scheduler: RefCell<Option<PendingSchedulerState>>,
    frame: RefCell<FrameState>,
}
// This is not necessarily true, but we are *always* executing in a
// single-threaded context (at least at the time of writing), so
// this is a happy little lie to avoid unnecessary locking
unsafe impl Sync for GlobalState {}
impl GlobalState {
    const fn new() -> Self {
        GlobalState { scheduler: RefCell::new(None), pending_scheduler: RefCell::new(None), frame: RefCell::new(FrameState::new()) }
    }

    fn initialize(&self) {
        *self.scheduler.borrow_mut() = Some(SchedulerState {
            callbacks: Default::default(),
            once_callbacks: Default::default(),
            futures: Default::default(),
            waker: unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &RAW_WAKER)) },
        });
        *self.pending_scheduler.borrow_mut() =
            Some(PendingSchedulerState { callbacks: Default::default(), once_callbacks: Default::default(), futures: Default::default() });
    }

    fn scheduler_mut(&self) -> RefMut<SchedulerState> {
        RefMut::map(self.scheduler.borrow_mut(), |r| r.as_mut().unwrap())
    }

    fn pending_scheduler_mut(&self) -> RefMut<PendingSchedulerState> {
        RefMut::map(self.pending_scheduler.borrow_mut(), |r| r.as_mut().unwrap())
    }

    fn frame(&self) -> Ref<FrameState> {
        self.frame.borrow()
    }
    fn frame_mut(&self) -> RefMut<FrameState> {
        self.frame.borrow_mut()
    }

    fn load_pending_into_current_scheduler(&self) {
        let (mut st, mut pst) = (self.scheduler_mut(), self.pending_scheduler_mut());
        for (event_name, mut new_callbacks) in pst.callbacks.drain() {
            st.callbacks.entry(event_name).or_default().append(&mut new_callbacks);
        }
        for (event_name, mut new_callbacks) in pst.once_callbacks.drain() {
            st.once_callbacks.entry(event_name).or_default().append(&mut new_callbacks);
        }
        st.futures.append(&mut pst.futures);
    }
}

static GLOBAL_STATE: GlobalState = GlobalState::new();

static RAW_WAKER: RawWakerVTable = RawWakerVTable::new(|_| RawWaker::new(std::ptr::null(), &RAW_WAKER), |_| {}, |_| {}, |_| {});

struct Guest;
impl guest::Guest for Guest {
    fn init() {
        GLOBAL_STATE.initialize();
    }

    fn exec(ctx: guest::RunContext, event_name: String, components: Vec<(u32, guest::ComponentType)>) {
        use guest_conversion::GuestConvert;

        let components = Components(components.into_iter().map(|(id, ct)| (id, ct.guest_convert())).collect());

        {
            let mut frame = GLOBAL_STATE.frame_mut();
            frame.time = ctx.time;
            frame.frametime = ctx.frametime;
        }

        {
            // Move all futures and callbacks from pending over to the main global state.
            GLOBAL_STATE.load_pending_into_current_scheduler();
        }

        {
            let mut global_state = GLOBAL_STATE.scheduler_mut();

            let mut futures = vec![];
            if let Some(callbacks) = global_state.callbacks.get(event_name.as_str()) {
                for callback in callbacks {
                    futures.push(callback(&components));
                }
            }

            for callback in global_state.once_callbacks.remove(event_name.as_str()).unwrap_or_default() {
                futures.push(callback(&components));
            }
            global_state.futures.append(&mut futures);

            global_state.execute_futures();
        }
    }
}

/// The time, relative to when the application started, in seconds.
/// This can be used to time how long something takes.
pub fn time() -> f32 {
    GLOBAL_STATE.frame().time
}

/// The length of the previous frame, in seconds.
pub fn frametime() -> f32 {
    GLOBAL_STATE.frame().frametime
}

/// `on` calls `callback` every time `event` occurs.
///
/// If you only want to be notified once, use [once].
///
/// The `callback` is a `fn`. This can be a closure (e.g. `|args| { ... }`).
pub fn on(event: &str, callback: impl Fn(&Components) -> EventResult + 'static) {
    on_async(event, move |args| std::future::ready(callback(args)))
}

/// `on_async` calls `callback` every time `event` occurs.
///
/// If you only want to be notified once, use [once_async].
///
/// The `callback` is a `async fn`. This can be a closure (e.g. `|args| { ... }`).
pub fn on_async<R: Future<Output = EventResult> + 'static>(event: &str, callback: impl Fn(&Components) -> R + 'static) {
    crate::host::event_subscribe(event);
    GLOBAL_STATE
        .pending_scheduler_mut()
        .callbacks
        .entry(event.to_string())
        .or_default()
        .push(Box::new(move |args| Box::pin(callback(args))));
}

/// `once` calls `callback` when `event` occurs, but only once.
///
/// If you want to be notified every time the `event` occurs, use [on].
///
/// The `callback` is a `fn`. This can be a closure (e.g. `|args| { ... }`).
pub fn once(event: &str, callback: impl FnOnce(&Components) -> EventResult + 'static) {
    once_async(event, |args| std::future::ready(callback(args)))
}

/// `once_async` calls `callback` when `event` occurs, but only once.
///
/// If you want to be notified every time the `event` occurs, use [on_async].
///
/// The `callback` is a `async fn`. This can be a closure (e.g. `|args| { ... }`).
pub fn once_async<R: Future<Output = EventResult> + 'static>(event: &str, callback: impl FnOnce(&Components) -> R + 'static) {
    crate::host::event_subscribe(event);
    GLOBAL_STATE
        .pending_scheduler_mut()
        .once_callbacks
        .entry(event.to_string())
        .or_default()
        .push(Box::new(move |args| Box::pin(callback(args))));
}

/// Runs the given async block (`future`). This lets your script set up behaviour
/// to run concurrently, like a long-running task.
///
/// This is similar to [tokio::spawn](https://docs.rs/tokio/latest/tokio/fn.spawn.html),
/// as well as similar functions from other async runtimes.
///
/// # Examples
/// ```
/// run_async(async {
///     notification::broadcast("a title", "hello!");
///     sleep(2.0).await;
///     notification::broadcast("a title", "hello to you too!");
///     EventOk
/// });
/// ```
pub fn run_async(future: impl Future<Output = EventResult> + 'static) {
    GLOBAL_STATE.pending_scheduler_mut().futures.push(Box::pin(future));
}
