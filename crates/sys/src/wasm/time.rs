use std::{
    mem,
    ops::{Add, Sub},
    time::Duration,
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
        Self(NotNan::new(*self.0 + rhs.as_nanos() as f64 / 1e6).unwrap())
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
        Self::now().duration_since(self)
    }

    #[inline]
    pub fn duration_since(&self, earlier: Self) -> Duration {
        Duration::from_nanos(((*self.0 - *earlier.0).max(0.0) * 1e6) as _)
    }
}

pub fn schedule_wakeup<F: 'static + Send + FnOnce()>(dur: Duration, callback: F) {
    let timer = gloo::timers::callback::Timeout::new(dur.as_millis().try_into().unwrap(), callback);
    mem::forget(timer);
}

use crate::timer::{get_global_timers, Sleep};
pub fn sleep_until(instant: Instant) -> Sleep {
    Sleep::new_at(&get_global_timers().expect("No timers"), instant)
}

pub fn sleep(dur: Duration) -> Sleep {
    Sleep::new(&get_global_timers().expect("No timers"), dur)
}
