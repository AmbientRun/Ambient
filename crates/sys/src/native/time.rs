#![allow(clippy::disallowed_types)]
use std::{
    ops::{Add, Sub},
    time::{Duration, SystemTime},
};

/// Represents an abstract point in time
/// This is intentionally a newtype to discourage mixing with StdInstant as it is not supported on
/// wasm.
#[derive(From, Into, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(pub(crate) std::time::Instant);

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0.checked_sub(rhs).unwrap())
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Instant {
    pub fn from_tokio(instant: tokio::time::Instant) -> Self {
        Self(instant.into())
    }

    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.0.duration_since(earlier.0)
    }
}

pub fn schedule_wakeup<F: 'static + Send + FnOnce()>(dur: Duration, callback: F) {
    tokio::spawn(async move {
        tokio::time::sleep(dur).await;
        callback()
    });
}

pub fn current_epoch_time() -> Duration {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap()
}

use derive_more::{From, Into};

use crate::MissedTickBehavior;

#[inline]
pub fn sleep_until(deadline: Instant) -> tokio::time::Sleep {
    tokio::time::sleep_until(deadline.0.into())
}

#[inline]
pub fn sleep(duration: Duration) -> tokio::time::Sleep {
    tokio::time::sleep(duration)
}

pub struct Interval {
    inner: tokio::time::Interval,
}

impl Interval {
    pub fn new(period: Duration) -> Self {
        Self::new_at(Instant::now(), period)
    }

    pub fn new_at(start: Instant, period: Duration) -> Self {
        Self { inner: tokio::time::interval_at(start.0.into(), period) }
    }

    pub async fn tick(&mut self) -> Instant {
        Instant::from_tokio(self.inner.tick().await)
    }

    pub fn set_missed_tick_behavior(&mut self, behavior: MissedTickBehavior) {
        self.inner.set_missed_tick_behavior(behavior.into())
    }
}
