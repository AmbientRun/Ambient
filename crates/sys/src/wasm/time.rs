use std::{
    mem,
    ops::{Add, Sub},
    time::{Duration, SystemTimeError},
};

use ordered_float::NotNan;

/// Represents an abstract point in time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(
    /// Time in milliseconds
    NotNan<f64>,
);

impl Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration::from_nanos(((*self.0 - *rhs.0).max(0.0) * 1e6) as _)
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.as_nanos() as f64 / 1e6)
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs.as_nanos() as f64 / 1e6)
    }
}

impl Instant {
    #[cfg(not(test))]
    pub fn now() -> Self {
        let perf = web_sys::window().unwrap().performance().unwrap();
        Self(NotNan::new(perf.now()).unwrap())
    }

    #[cfg(test)]
    pub fn now() -> Self {
        Self(Default::default())
    }

    pub fn elapsed(&self) -> Duration {
        Self::now().duration_since(*self)
    }

    #[inline]
    pub fn duration_since(&self, earlier: Self) -> Duration {
        Duration::from_nanos(((*self.0 - *earlier.0).max(0.0) * 1e6) as _)
    }
}

/// Measurement of the system clock
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(NotNan<f64>);

impl Add<Duration> for SystemTime {
    type Output = SystemTime;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.as_millis() as f64)
    }
}

impl Sub<Duration> for SystemTime {
    type Output = SystemTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs.as_millis() as f64)
    }
}

impl SystemTime {
    pub const UNIX_EPOCH: Self = SystemTime(unsafe { NotNan::new_unchecked(0.0) });

    pub fn now() -> Self {
        Self(NotNan::new(js_sys::Date::now()).unwrap())
    }

    pub fn duration_since(&self, earlier: Self) -> Result<Duration, SystemTimeError> {
        Ok(Duration::from_nanos(((*self.0 - *earlier.0).max(0.0) * 1e6) as _))
    }
}

pub fn schedule_wakeup<F: 'static + Send + FnOnce()>(dur: Duration, callback: F) {
    let timer = gloo::timers::callback::Timeout::new(dur.as_millis().try_into().unwrap(), callback);
    mem::forget(timer);
}

use crate::{
    timer::{self, get_global_timers, Sleep},
    MissedTickBehavior,
};
pub fn sleep_until(instant: Instant) -> Sleep {
    Sleep::new_at(&get_global_timers().expect("No timers"), instant)
}

pub fn sleep(dur: Duration) -> Sleep {
    Sleep::new(&get_global_timers().expect("No timers"), dur)
}

pub struct Interval {
    inner: timer::Interval,
}

impl Interval {
    pub fn new(period: Duration) -> Self {
        Self::new_at(Instant::now(), period)
    }

    pub fn new_at(start: Instant, period: Duration) -> Self {
        Self { inner: timer::Interval::new_at(&get_global_timers().expect("No timers"), start, period) }
    }

    pub async fn tick(&mut self) -> Instant {
        self.inner.tick().await
    }

    pub fn set_missed_tick_behavior(&mut self, behavior: MissedTickBehavior) {
        self.inner.set_missed_tick_behavior(behavior)
    }
}
