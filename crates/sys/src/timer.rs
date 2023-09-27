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

use futures::{ready, FutureExt, Stream, StreamExt};
use parking_lot::Mutex;
use slotmap::SlotMap;

use crate::{time::Instant, MissedTickBehavior};

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
        self.woken
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
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
    /// A waker for the update future when a new timer is added.
    waiter: Waiter,
    inner: Mutex<Inner>,
}

impl TimerStore {
    fn update_timer(&self, key: TimerKey, deadline: Instant) {
        let mut guard = self.inner.lock();
        let timer = guard.timers.get_mut(key).expect("Invalid timer handle");
        timer.deadline = deadline;

        guard.heap.push((Reverse(deadline), key));

        drop(guard);

        self.waiter.wake();
    }

    /// Creates and starts a new timer
    fn new_timer(&self, deadline: Instant) -> (TimerKey, Arc<Waiter>) {
        let waiter = Arc::new(Waiter::default());

        let timer = Timer {
            deadline,
            waiter: waiter.clone(),
        };

        let mut guard = self.inner.lock();
        let key = guard.timers.insert(timer);

        guard.heap.push((Reverse(deadline), key));
        drop(guard);

        self.waiter.wake();

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
        let mut store = self.timers.inner.lock();
        let store = &mut *store;

        tracing::debug!("Updating timers: {}", store.heap.len());
        let mut dur = None;
        while let Some(v) = store.heap.peek_mut() {
            // The deadline stored in the key may be out of date and only serves as a hint
            let key = v.1;
            let Some(timer) = store.timers.get_mut(key) else {
                PeekMut::pop(v);
                continue;
            };

            if timer.deadline <= now {
                timer.waiter.wake();

                PeekMut::pop(v);
            } else {
                // Timer has not yet expired

                dur = Some(timer.deadline - now);
                break;
            }
        }

        if let Some(dur) = dur {
            // The waker will be filled in when the executor polls the returned future.
            let timers = self.timers.clone();

            crate::time::schedule_wakeup(dur, move || {
                timers.waiter.wake();
            });
        }

        TimerWheelUpdate {
            waiter: &self.timers.waiter,
        }
    }

    pub fn timers(&self) -> &Arc<TimerStore> {
        &self.timers
    }
}

pub struct TimerWheelUpdate<'a> {
    waiter: &'a Waiter,
}

impl<'a> Future for TimerWheelUpdate<'a> {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
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
        Self {
            key,
            timers: Arc::downgrade(timers),
            waiter,
        }
    }

    pub fn new(timers: &Arc<TimerStore>, dur: Duration) -> Self {
        let deadline = Instant::now() + dur;
        let (key, waiter) = timers.new_timer(deadline);
        Self {
            key,
            timers: Arc::downgrade(timers),
            waiter,
        }
    }

    /// Resets the sleep future to a new deadline even if expired
    pub fn reset(&mut self, deadline: Instant) {
        if let Some(timers) = self.timers.upgrade() {
            timers.update_timer(self.key, deadline)
        } else {
            panic!("No timers");
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if let Some(_timers) = self.timers.upgrade() {
            if self.waiter.take_woken() {
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

pub struct Interval {
    fut: Sleep,
    deadline: Instant,
    period: Duration,
    behavior: MissedTickBehavior,
}

impl Interval {
    pub fn new(timers: &Arc<TimerStore>, period: Duration) -> Self {
        Self::new_at(timers, Instant::now(), period)
    }

    pub fn new_at(timers: &Arc<TimerStore>, deadline: Instant, period: Duration) -> Self {
        Self {
            fut: Sleep::new_at(timers, deadline),
            deadline,
            period,
            behavior: Default::default(),
        }
    }

    pub async fn tick(&mut self) -> Instant {
        self.next().await.unwrap()
    }

    pub fn set_missed_tick_behavior(&mut self, behavior: MissedTickBehavior) {
        self.behavior = behavior;
    }
}

impl Stream for Interval {
    type Item = Instant;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Wait for the next deadline
        ready!(self.fut.poll_unpin(cx));

        let now = Instant::now();
        let next_deadline = self.behavior.next_timeout(self.deadline, now, self.period);
        self.deadline = next_deadline;
        self.fut.reset(next_deadline);

        Poll::Ready(Some(now))
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
mod test {
    use crate::time::sleep;

    use super::*;

    async fn assert_dur(fut: impl Future, dur: Duration) {
        let now = Instant::now();
        fut.await;

        let elapsed = now.elapsed();
        assert!(
            (elapsed.as_secs_f32() - dur.as_secs_f32()).abs() < dur.as_secs_f32().max(0.1) * 0.1,
            "Expected future to take {dur:?} but it took {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn interval_skip() {
        let wheel = TimerWheel::new();
        let timers = wheel.timers();

        let mut interval = Interval::new(timers, Duration::from_millis(500));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        tokio::spawn(wheel.start());

        assert_dur(interval.tick(), Duration::ZERO).await;
        assert_dur(interval.tick(), Duration::from_millis(500)).await;
        // assert_dur(interval.tick(), Duration::from_millis(500)).await;

        assert_dur(
            sleep(Duration::from_millis(1250)),
            Duration::from_millis(1250),
        )
        .await;

        assert_dur(interval.tick(), Duration::ZERO).await;

        // Oh no, we missed a tick
        // Fire at 1500
        assert_dur(interval.tick(), Duration::from_millis(250)).await;
        assert_dur(interval.tick(), Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn interval() {
        let wheel = TimerWheel::new();
        let timers = wheel.timers();

        let mut interval = Interval::new(timers, Duration::from_millis(500));
        tokio::spawn(wheel.start());

        assert_dur(interval.tick(), Duration::ZERO).await;
        assert_dur(interval.tick(), Duration::from_millis(500)).await;
        // assert_dur(interval.tick(), Duration::from_millis(500)).await;

        assert_dur(
            sleep(Duration::from_millis(1250)),
            Duration::from_millis(1250),
        )
        .await;

        // Resolves immediately, now 750 ms behind
        assert_dur(interval.tick(), Duration::ZERO).await;
        // 250 ms
        assert_dur(interval.tick(), Duration::ZERO).await;

        assert_dur(interval.tick(), Duration::from_millis(250)).await;
        // assert_dur(interval.tick(), Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn timer_reset() {
        let wheel = TimerWheel::new();
        let timers = wheel.timers();

        let mut fut = Sleep::new(timers, Duration::from_secs(5));
        fut.reset(Instant::now() + Duration::from_secs(1));

        tokio::spawn(wheel.start());

        assert_dur(&mut fut, Duration::from_secs(1)).await;

        fut.reset(Instant::now() + Duration::from_secs(2));

        assert_dur(&mut fut, Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
#[cfg(target_os = "unknown")]
mod test {

    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn timers() {
        let now = Instant::now();
        eprintln!("Now: {now:?}");
    }
}
