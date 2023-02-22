use std::{
    ops::{Add, Sub},
    time::Duration,
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

impl Add<Instant> for Instant {
    type Output = Duration;

    fn add(self, rhs: Instant) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Instant {
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

use derive_more::{From, Into};
pub use tokio::time::{sleep, sleep_until};
