use std::{
    cmp::Reverse,
    collections::{binary_heap::PeekMut, BinaryHeap},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Weak,
    },
    task::{Poll, Waker},
    time::Duration,
};

use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::time::Instant;

/// Represents a timer which will be invoked at a point in time
#[derive(Debug)]
pub struct Timer {
    /// When does this timer expire
    deadline: Instant,
    waiter: Arc<Waiter>,
}

slotmap::new_key_type! {
    pub struct TimerKey;
}

#[derive(Default, Debug)]
struct Waiter {
    woken: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl Waiter {
    fn take_woken(&self) -> bool {
        self.woken.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }

    fn wake(&self) {
        self.woken.store(true, Ordering::SeqCst);
        if let Some(waker) = &mut *self.waker.lock() {
            waker.wake_by_ref();
        }
    }
}

#[derive(Default, Debug)]
struct Inner {
    timers: SlotMap<TimerKey, Timer>,
    /// Store the timers closest in time
    heap: BinaryHeap<(Reverse<Instant>, TimerKey)>,
}

#[derive(Default, Debug)]
pub struct TimerStore {
    inner: Mutex<Inner>,
}

impl TimerStore {
    /// Creates and starts a new timer
    fn new_timer(&self, deadline: Instant) -> (TimerKey, Arc<Waiter>) {
        let waiter = Arc::new(Waiter::default());

        let timer = Timer { deadline, waiter: waiter.clone() };

        let mut inner = self.inner.lock();
        let key = inner.timers.insert(timer);

        inner.heap.push((Reverse(deadline), key));

        (key, waiter)
    }
}

static TIMERS: Mutex<Option<Weak<TimerStore>>> = Mutex::new(None);
/// Retrieve the global timer store

pub fn get_global_timers() -> Option<Arc<TimerStore>> {
    Weak::upgrade(TIMERS.lock().as_ref()?)
}

#[derive(Default, Debug)]
pub struct TimerWheel {
    timers: Arc<TimerStore>,
    /// A waker for the update future when a new timer is added.
    waiter: Arc<Waiter>,
}

impl TimerWheel {
    pub fn new() -> Self {
        Default::default()
    }

    /// Executes as a global timer wheel.
    ///
    /// The future must be spawned or polled
    pub fn start(mut self) -> impl Future<Output = ()> {
        *TIMERS.lock() = Some(Arc::downgrade(&self.timers));

        async move {
            loop {
                let now = Instant::now();
                self.update(now).await;
            }
        }
    }

    /// Updates the timer wheel and completed the appropriate timers.
    ///
    /// Returns the duration to schedule the next update.
    pub fn update(&mut self, now: Instant) -> TimerWheelUpdate {
        tracing::info!("Updating timers");
        let mut store = self.timers.inner.lock();
        let store = &mut *store;

        let mut dur = None;
        while let Some(v) = store.heap.peek_mut() {
            let key = v.1;
            let Some(timer) = store.timers.get_mut(key) else {
                tracing::info!("Timer was removed");
                PeekMut::pop(v);
                continue;
            };

            assert_eq!(v.0 .0, timer.deadline);

            if timer.deadline <= now {
                tracing::info!("Timer {key:?} expired");
                timer.waiter.wake();

                PeekMut::pop(v);
            } else {
                // Timer has not yet expired

                dur = Some(timer.deadline - now);
                tracing::info!(?now, deadline = ?timer.deadline, "Next timer: {dur:?}");
                break;
            }
        }

        if let Some(dur) = dur {
            // The waker will be filled in when the executor polls the returned future.
            let waiter = self.waiter.clone();
            tracing::info!("Scheduling a wakeup for {dur:?}");

            crate::time::schedule_wakeup(dur, move || {
                waiter.wake();
            });
        }

        TimerWheelUpdate { waiter: &self.waiter }
    }
}

pub struct TimerWheelUpdate<'a> {
    waiter: &'a Waiter,
}

impl<'a> Future for TimerWheelUpdate<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if self.waiter.take_woken() {
            Poll::Ready(())
        } else {
            // Will be woken by `schedule_wakeup` or pushing a new timer
            *self.waiter.waker.lock() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Sleep future
#[must_use]
pub struct Sleep {
    key: TimerKey,
    waiter: Arc<Waiter>,
    timers: Weak<TimerStore>,
}

impl Sleep {
    pub fn new_at(timers: &Arc<TimerStore>, deadline: Instant) -> Self {
        let (key, waiter) = timers.new_timer(deadline);
        Self { key, timers: Arc::downgrade(timers), waiter }
    }

    pub fn new(timers: &Arc<TimerStore>, dur: Duration) -> Self {
        let deadline = Instant::now() + dur;
        let (key, waiter) = timers.new_timer(deadline);
        Self { key, timers: Arc::downgrade(timers), waiter }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if let Some(timers) = self.timers.upgrade() {
            if self.waiter.take_woken() {
                let mut timers = timers.inner.lock();
                timers.timers.remove(self.key);

                Poll::Ready(())
            } else {
                *self.waiter.waker.lock() = Some(cx.waker().clone());
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let Some(timers) = self.timers.upgrade() {
            let mut timers = timers.inner.lock();
            timers.timers.remove(self.key);
        }
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod test {

    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn timers() {
        let now = Instant::now();
        eprintln!("Now: {now:?}");
    }
}
