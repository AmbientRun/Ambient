//! This implementation of missing tick behavior and backpressure are included from [`tokio`](https://github.com/tokio-rs/tokio).
//!
//! The original license is as follows:
//!
//! Copyright (c) 2023 Tokio Contributors
//!
//! Permission is hereby granted, free of charge, to any
//! person obtaining a copy of this software and associated
//! documentation files (the "Software"), to deal in the
//! Software without restriction, including without
//! limitation the rights to use, copy, modify, merge,
//! publish, distribute, sublicense, and/or sell copies of
//! the Software, and to permit persons to whom the Software
//! is furnished to do so, subject to the following
//! conditions:
//!
//! The above copyright notice and this permission notice
//! shall be included in all copies or substantial portions
//! of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//! ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//! TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//! PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//! SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//! OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//! IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.

use std::time::Duration;

#[allow(unused_imports)]
use crate::platform::time::{Instant, Interval};

/// Defines the behavior of an [`Interval`] when it misses a tick.
///
/// Sometimes, an [`Interval`]'s tick is missed. For example, consider the
/// following:
///
/// ```
/// use ambient_sys::time::{self, Duration};
/// # async fn task_that_takes_one_to_three_millis() {}
///
/// #[tokio::main]
/// async fn main() {
///     // ticks every 2 milliseconds
///     let mut interval = time::interval(Duration::from_millis(2));
///     for _ in 0..5 {
///         interval.tick().await;
///         // if this takes more than 2 milliseconds, a tick will be delayed
///         task_that_takes_one_to_three_millis().await;
///     }
/// }
/// ```
///
/// Generally, a tick is missed if too much time is spent without calling
/// [`Interval::tick()`].
///
/// By default, when a tick is missed, [`Interval`] fires ticks as quickly as it
/// can until it is "caught up" in time to where it should be.
/// `MissedTickBehavior` can be used to specify a different behavior for
/// [`Interval`] to exhibit. Each variant represents a different strategy.
///
/// Note that because the executor cannot guarantee exact precision with timers,
/// these strategies will only apply when the delay is greater than 5
/// milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissedTickBehavior {
    /// Ticks as fast as possible until caught up.
    ///
    /// When this strategy is used, [`Interval`] schedules ticks "normally" (the
    /// same as it would have if the ticks hadn't been delayed), which results
    /// in it firing ticks as fast as possible until it is caught up in time to
    /// where it should be. Unlike [`Delay`] and [`Skip`], the ticks yielded
    /// when `Burst` is used (the [`Instant`]s that [`tick`](Interval::tick)
    /// yields) aren't different than they would have been if a tick had not
    /// been missed. Like [`Skip`], and unlike [`Delay`], the ticks may be
    /// shortened.
    ///
    /// This looks something like this:
    /// ```text
    /// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
    /// Actual ticks:   | work -----|          delay          | work | work | work -| work -----|
    /// ```
    ///
    /// In code:
    ///
    /// ```
    /// use ambient_sys::time::{interval, Duration};
    /// # async fn task_that_takes_200_millis() {}
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let mut interval = interval(Duration::from_millis(50));
    ///
    /// // First tick resolves immediately after creation
    /// interval.tick().await;
    ///
    /// task_that_takes_200_millis().await;
    /// // The `Interval` has missed a tick
    ///
    /// // Since we have exceeded our timeout, this will resolve immediately
    /// interval.tick().await;
    ///
    /// // Since we are more than 100ms after the start of `interval`, this will
    /// // also resolve immediately.
    /// interval.tick().await;
    ///
    /// // Also resolves immediately, because it was supposed to resolve at
    /// // 150ms after the start of `interval`
    /// interval.tick().await;
    ///
    /// // Resolves immediately
    /// interval.tick().await;
    ///
    /// // Since we have gotten to 200ms after the start of `interval`, this
    /// // will resolve after 50ms
    /// interval.tick().await;
    /// # }
    /// ```
    ///
    /// This is the default behavior when [`Interval`] is created with
    /// `interval` and `interval_at`.
    ///
    /// [`Delay`]: MissedTickBehavior::Delay
    /// [`Skip`]: MissedTickBehavior::Skip
    Burst,

    /// Tick at multiples of `period` from when [`tick`] was called, rather than
    /// from `start`.
    ///
    /// When this strategy is used and [`Interval`] has missed a tick, instead
    /// of scheduling ticks to fire at multiples of `period` from `start` (the
    /// time when the first tick was fired), it schedules all future ticks to
    /// happen at a regular `period` from the point when [`tick`] was called.
    /// Unlike [`Burst`] and [`Skip`], ticks are not shortened, and they aren't
    /// guaranteed to happen at a multiple of `period` from `start` any longer.
    ///
    /// This looks something like this:
    /// ```text
    /// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
    /// Actual ticks:   | work -----|          delay          | work -----| work -----| work -----|
    /// ```
    ///
    /// In code:
    ///
    /// ```
    /// use ambient_sys::time::{interval, Duration, MissedTickBehavior};
    /// # async fn task_that_takes_more_than_50_millis() {}
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let mut interval = interval(Duration::from_millis(50));
    /// interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ///
    /// task_that_takes_more_than_50_millis().await;
    /// // The `Interval` has missed a tick
    ///
    /// // Since we have exceeded our timeout, this will resolve immediately
    /// interval.tick().await;
    ///
    /// // But this one, rather than also resolving immediately, as might happen
    /// // with the `Burst` or `Skip` behaviors, will not resolve until
    /// // 50ms after the call to `tick` up above. That is, in `tick`, when we
    /// // recognize that we missed a tick, we schedule the next tick to happen
    /// // 50ms (or whatever the `period` is) from right then, not from when
    /// // were *supposed* to tick
    /// interval.tick().await;
    /// # }
    /// ```
    ///
    /// [`Burst`]: MissedTickBehavior::Burst
    /// [`Skip`]: MissedTickBehavior::Skip
    /// [`tick`]: Interval::tick
    Delay,

    /// Skips missed ticks and tick on the next multiple of `period` from
    /// `start`.
    ///
    /// When this strategy is used, [`Interval`] schedules the next tick to fire
    /// at the next-closest tick that is a multiple of `period` away from
    /// `start` (the point where [`Interval`] first ticked). Like [`Burst`], all
    /// ticks remain multiples of `period` away from `start`, but unlike
    /// [`Burst`], the ticks may not be *one* multiple of `period` away from the
    /// last tick. Like [`Delay`], the ticks are no longer the same as they
    /// would have been if ticks had not been missed, but unlike [`Delay`], and
    /// like [`Burst`], the ticks may be shortened to be less than one `period`
    /// away from each other.
    ///
    /// This looks something like this:
    /// ```text
    /// Expected ticks: |     1     |     2     |     3     |     4     |     5     |     6     |
    /// Actual ticks:   | work -----|          delay          | work ---| work -----| work -----|
    /// ```
    ///
    /// In code:
    ///
    /// ```
    /// use ambient_sys::time::{interval, Duration, MissedTickBehavior};
    /// # async fn task_that_takes_75_millis() {}
    ///
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() {
    /// let mut interval = interval(Duration::from_millis(50));
    /// interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    ///
    /// task_that_takes_75_millis().await;
    /// // The `Interval` has missed a tick
    ///
    /// // Since we have exceeded our timeout, this will resolve immediately
    /// interval.tick().await;
    ///
    /// // This one will resolve after 25ms, 100ms after the start of
    /// // `interval`, which is the closest multiple of `period` from the start
    /// // of `interval` after the call to `tick` up above.
    /// interval.tick().await;
    /// # }
    /// ```
    ///
    /// [`Burst`]: MissedTickBehavior::Burst
    /// [`Delay`]: MissedTickBehavior::Delay
    Skip,
}

impl MissedTickBehavior {
    /// If a tick is missed, this method is called to determine when the next tick should happen.
    pub(crate) fn next_timeout(&self, timeout: Instant, now: Instant, period: Duration) -> Instant {
        match self {
            Self::Burst => timeout + period,
            Self::Delay => now + period,
            Self::Skip => {
                now + period
                    - Duration::from_nanos(
                        ((now - timeout).as_nanos() % period.as_nanos())
                            .try_into()
                            // This operation is practically guaranteed not to
                            // fail, as in order for it to fail, `period` would
                            // have to be longer than `now - timeout`, and both
                            // would have to be longer than 584 years.
                            //
                            // If it did fail, there's not a good way to pass
                            // the error along to the user, so we just panic.
                            .expect(
                                "too much time has elapsed since the interval was supposed to tick",
                            ),
                    )
            }
        }
    }
}

impl Default for MissedTickBehavior {
    /// Returns [`MissedTickBehavior::Burst`].
    ///
    /// For most usecases, the [`Burst`] strategy is what is desired.
    /// Additionally, to preserve backwards compatibility, the [`Burst`]
    /// strategy must be the default. For these reasons,
    /// [`MissedTickBehavior::Burst`] is the default for [`MissedTickBehavior`].
    /// See [`Burst`] for more details.
    ///
    /// [`Burst`]: MissedTickBehavior::Burst
    fn default() -> Self {
        Self::Burst
    }
}

#[cfg(not(target_os = "unknown"))]
impl From<MissedTickBehavior> for tokio::time::MissedTickBehavior {
    fn from(value: MissedTickBehavior) -> Self {
        match value {
            MissedTickBehavior::Burst => tokio::time::MissedTickBehavior::Burst,
            MissedTickBehavior::Delay => tokio::time::MissedTickBehavior::Delay,
            MissedTickBehavior::Skip => tokio::time::MissedTickBehavior::Skip,
        }
    }
}
