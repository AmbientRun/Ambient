use std::{
    collections::BTreeSet,
    marker::PhantomPinned,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll, Waker},
    thread::{self, Thread},
    time::Duration,
};

use futures::{
    task::{ArcWake, AtomicWaker},
    Future,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};
use slotmap::new_key_type;
mod interval;

pub use crate::platform::time::{Instant, SystemTime};

pub use interval::{interval, interval_at, Interval};

pub static GLOBAL_TIMER: Lazy<TimersHandle> = Lazy::new(Timers::start);

// #[deprecated(note = "use labeled sleep")]
// pub fn sleep_until(deadline: Instant) -> Sleep {
//     Sleep::new(&GLOBAL_TIMER, deadline, "unknown until")
// }

// #[deprecated(note = "use labeled sleep")]
// pub fn sleep(duration: Duration) -> Sleep {
//     Sleep::new(&GLOBAL_TIMER, Instant::now() + duration, "unknown")
// }

pub fn sleep_label(duration: Duration, label: &'static str) -> Sleep {
    tracing::info!(?duration, "sleep");
    Sleep::new(&GLOBAL_TIMER, Instant::now() + duration, label)
}

struct TimerEntry {
    waker: AtomicWaker,
    finished: AtomicBool,
    _pinned: PhantomPinned,
    label: &'static str,
}

new_key_type! {
    pub struct TimerKey;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct Entry {
    deadline: Instant,
    timer: *const TimerEntry,
}

unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

struct ThreadWaker {
    thread_id: Thread,
}

impl ArcWake for ThreadWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.thread_id.unpark()
    }
}

struct Inner {
    /// Invoked when there is a new timer
    waker: AtomicWaker,
    heap: Mutex<BTreeSet<Entry>>,
    handle_count: AtomicUsize,
}

impl Inner {
    pub fn register(&self, deadline: Instant, timer: *const TimerEntry) {
        tracing::info!(?deadline, "register timer");
        self.heap.lock().insert(Entry { deadline, timer });

        self.waker.wake();
    }

    fn remove(&self, deadline: Instant, timer: *const TimerEntry) {
        tracing::info!(?deadline, "remove timer");
        let removed = self.heap.lock().remove(&Entry { deadline, timer });
        let label = unsafe { &*timer }.label;
        if removed {
            tracing::info!(label, "removed timer");
        } else {
            tracing::info!(label, "timer not present to remove");
        }
    }
}

pub struct TimersHandle {
    inner: Arc<Inner>,
}

impl Clone for TimersHandle {
    fn clone(&self) -> Self {
        self.inner.handle_count.fetch_add(1, Ordering::Relaxed);

        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Drop for TimersHandle {
    fn drop(&mut self) {
        let count = self.inner.handle_count.fetch_sub(1, Ordering::Relaxed);
        if count == 1 {
            self.inner.waker.wake();
        }
    }
}

pub struct Timers {
    inner: Arc<Inner>,
}

pub struct TimersFinished;

impl Timers {
    pub fn new() -> (Self, TimersHandle) {
        let inner = Arc::new(Inner {
            heap: Mutex::new(BTreeSet::new()),
            waker: AtomicWaker::new(),
            handle_count: AtomicUsize::new(1),
        });

        (
            Self {
                inner: inner.clone(),
            },
            TimersHandle { inner },
        )
    }

    /// Advances the timers, returning the next deadline
    fn tick(&mut self, time: Instant, waker: &Waker) -> Result<Option<Instant>, TimersFinished> {
        self.inner.waker.register(waker);

        let mut heap = self.inner.heap.lock();

        tracing::debug!(count = heap.len(), "Timers::tick");
        while let Some(entry) = heap.first() {
            // All deadlines before now have been handled
            if entry.deadline > time {
                return Ok(Some(entry.deadline));
            }

            let entry = heap.pop_first().unwrap();
            tracing::info!(?entry.deadline, "popped timer");
            // Fire and wake the timer
            // # Safety
            // Sleep removes the timer when dropped
            // Drop is guaranteed due to Sleep being pinned when registered
            let timer = unsafe { &*(entry.timer) };

            tracing::info!(label = timer.label, "timer fired");

            // Wake the future waiting on the timer
            timer.finished.store(true, Ordering::SeqCst);
            timer.waker.wake();
        }

        if self.inner.handle_count.load(Ordering::SeqCst) == 0 {
            return Err(TimersFinished);
        }

        Ok(None)
    }

    /// Starts executing the timers in the background
    pub fn start() -> TimersHandle {
        let (timers, handle) = Timers::new();
        #[cfg(target_os = "unknown")]
        wasm_bindgen_futures::spawn_local(timers.run_wasm());

        #[cfg(not(target_os = "unknown"))]
        std::thread::spawn(move || timers.run_blocking());

        handle
    }

    pub fn run_blocking(mut self) {
        let waker = Arc::new(ThreadWaker {
            thread_id: thread::current(),
        });

        let waker = futures::task::waker(waker);

        loop {
            let now = Instant::now();
            let next = match self.tick(now, &waker) {
                Ok(v) => v,
                Err(_) => {
                    break;
                }
            };

            if let Some(next) = next {
                let dur = next - now;
                thread::park_timeout(dur)
            } else {
                thread::park();
            }
        }
    }

    #[cfg(target_os = "unknown")]
    pub fn run_wasm(mut self) -> impl Future<Output = ()> {
        let reactor = AsyncReactor {
            timers: self,
            timeout: None,
        };

        reactor
    }
}

#[cfg(target_os = "unknown")]
struct AsyncReactor {
    timers: Timers,
    timeout: Option<gloo_timers::callback::Timeout>,
}

#[cfg(target_os = "unknown")]
impl Future for AsyncReactor {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = Instant::now();

        let next = match self.timers.tick(now, cx.waker()) {
            Ok(v) => v,
            Err(_) => {
                tracing::info!("No more timers");
                return Poll::Ready(());
            }
        };

        if let Some(next) = next {
            let dur = next - now;
            tracing::debug!(?dur, "schedule wakeup");

            let waker = cx.waker().clone();
            let timer = gloo_timers::callback::Timeout::new(
                dur.as_millis().try_into().unwrap(),
                move || {
                    tracing::debug!("wake after timeout");
                    waker.wake_by_ref();
                },
            );

            self.timeout = Some(timer);

            Poll::Pending
        } else {
            tracing::debug!("no timers, yield until a new timer is added");
            Poll::Pending
        }
    }
}

#[pin_project(PinnedDrop)]
/// Sleep future
pub struct Sleep {
    shared: Arc<Inner>,
    timer: Box<TimerEntry>,
    deadline: Instant,
    registered: bool,
    label: &'static str,
}

impl std::fmt::Debug for Sleep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sleep")
            .field("deadline", &self.deadline)
            .finish()
    }
}

impl Sleep {
    pub(crate) fn new(handle: &TimersHandle, deadline: Instant, label: &'static str) -> Self {
        tracing::info!(?label, ?deadline, "Sleep::new");
        Self {
            shared: handle.inner.clone(),
            timer: Box::new(TimerEntry {
                waker: AtomicWaker::new(),
                finished: AtomicBool::new(false),
                _pinned: PhantomPinned,
                label,
            }),
            deadline,
            registered: false,
            label,
        }
    }

    /// Set the label
    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = label;
        self
    }

    pub fn reset(self: Pin<&mut Self>, deadline: Instant) {
        let (timer, cur_deadline) = self.unregister();
        *cur_deadline = deadline;
        timer.finished.store(false, Ordering::SeqCst);
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    /// Removes the timer entry from the timers queue.
    ///
    /// The TimerEntry is no longer aliased and is safe to modify.
    fn unregister(self: Pin<&mut Self>) -> (&mut TimerEntry, &mut Instant) {
        let p = self.project();
        // This removes any existing reference to the TimerEntry pointer
        let shared = p.shared;
        shared.remove(*p.deadline, &**p.timer);

        *p.registered = false;
        (p.timer, p.deadline)
    }

    fn register_deadline(self: Pin<&mut Self>) {
        let p = self.project();
        p.shared.register(*p.deadline, &**p.timer);
        *p.registered = true;
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self
            .timer
            .finished
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            Poll::Ready(())
        } else if !self.registered {
            self.timer.waker.register(cx.waker());
            self.register_deadline();

            Poll::Pending
        } else {
            self.timer.waker.register(cx.waker());
            Poll::Pending
        }
    }
}

#[pinned_drop]
impl PinnedDrop for Sleep {
    fn drop(self: Pin<&mut Self>) {
        tracing::debug!(self.label, self.registered, "dropping sleep");

        if self.registered {
            self.unregister();
        }
    }
}

#[cfg(test)]
pub(crate) fn assert_dur(found: Duration, expected: Duration, msg: &str) {
    assert!(
        (found.as_millis().abs_diff(expected.as_millis())) < 10,
        "Expected {found:?} to be close to {expected:?}\n{msg}",
    )
}

#[cfg(test)]
fn setup_timers() -> (TimersHandle, thread::JoinHandle<()>) {
    let (timers, handle) = Timers::new();

    let thread = thread::Builder::new()
        .name("Timer".into())
        .spawn(move || timers.run_blocking())
        .unwrap();

    (handle, thread)
}

#[cfg(test)]
mod test {
    use std::{eprintln, time::Duration};

    use futures::{stream, FutureExt, StreamExt};

    use super::*;

    #[test]
    fn sleep() {
        let (handle, j) = setup_timers();
        let now = Instant::now();
        futures::executor::block_on(async move {
            Sleep::new(&handle, Instant::now() + Duration::from_millis(500), "a").await;

            eprintln!("Timer 1 finished");

            let now = Instant::now();
            Sleep::new(&handle, Instant::now() + Duration::from_millis(1000), "b").await;

            Sleep::new(&handle, now - Duration::from_millis(100), "b").await;

            eprintln!("Expired timer finished")
        });

        #[cfg(not(miri))]
        assert_dur(now.elapsed(), Duration::from_millis(500 + 1000), "seq");
        j.join().unwrap();
    }

    #[test]
    fn sleep_join() {
        let (handle, j) = setup_timers();

        let now = Instant::now();
        futures::executor::block_on(async move {
            let sleep_1 = Sleep::new(&handle, Instant::now() + Duration::from_millis(500), "a");

            eprintln!("Timer 1 finished");

            let now = Instant::now();
            let sleep_2 = Sleep::new(&handle, Instant::now() + Duration::from_millis(1000), "b");

            let sleep_3 = Sleep::new(&handle, now - Duration::from_millis(100), "c");

            futures::join!(sleep_1, sleep_2, sleep_3);

            eprintln!("Expired timer finished")
        });

        #[cfg(not(miri))]
        assert_dur(now.elapsed(), Duration::from_millis(1000), "join");
        j.join().unwrap();
    }

    #[test]
    fn sleep_race() {
        let (handle, j) = setup_timers();

        let now = Instant::now();
        futures::executor::block_on(async move {
            {
                let mut sleep_1 =
                    Sleep::new(&handle, Instant::now() + Duration::from_millis(500), "a").fuse();

                eprintln!("Timer 1 finished");

                let mut sleep_2 =
                    Sleep::new(&handle, Instant::now() + Duration::from_millis(1000), "b").fuse();

                futures::select!(_ = sleep_1 => {}, _ = sleep_2 => {});
            }

            Sleep::new(&handle, Instant::now() + Duration::from_millis(1500), "c").await;

            let _never_polled =
                Sleep::new(&handle, Instant::now() + Duration::from_millis(2000), "d");
            futures::pin_mut!(_never_polled);
        });

        #[cfg(not(miri))]
        assert_dur(now.elapsed(), Duration::from_millis(2000), "race");
        j.join().unwrap();
    }

    #[test]
    fn sleep_identical() {
        let (handle, j) = setup_timers();

        let now = Instant::now();
        futures::executor::block_on(async move {
            let deadline = now + Duration::from_millis(500);
            stream::iter(
                (0..100)
                    .map(|_| Sleep::new(&handle, deadline, "a"))
                    .collect::<Vec<_>>(),
            )
            .buffered(2048)
            .for_each(|_| async {
                // Sleep::new(&handle, Instant::now() + Duration::from_millis(100)).await;
            })
            .await;
        });

        #[cfg(not(miri))]
        assert_dur(now.elapsed(), Duration::from_millis(500), "seq");
        j.join().unwrap();
    }

    #[test]
    fn sleep_reset() {
        let (handle, j) = setup_timers();

        let now = Instant::now();
        futures::executor::block_on(async move {
            let sleep = Sleep::new(&handle, Instant::now() + Duration::from_millis(500), "a");

            futures::pin_mut!(sleep);
            sleep.as_mut().await;

            sleep
                .as_mut()
                .reset(Instant::now() + Duration::from_millis(1000));

            sleep.as_mut().await;
        });

        #[cfg(not(miri))]
        assert_dur(now.elapsed(), Duration::from_millis(1500), "seq");
        j.join().unwrap();
    }
}
