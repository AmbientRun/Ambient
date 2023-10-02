use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{ready, Future, Stream};
use pin_project::pin_project;

use crate::{
    time::{Sleep, TimersHandle, GLOBAL_TIMER},
    MissedTickBehavior,
};

use super::Instant;

pub fn interval(period: Duration) -> Interval {
    Interval::new(&GLOBAL_TIMER, Instant::now(), period, "unknown interval")
}

pub fn interval_at(start: Instant, period: Duration) -> Interval {
    Interval::new(&GLOBAL_TIMER, start, period, "unknown interval")
}

/// Ticks at a fixed interval.
#[pin_project]
#[derive(Debug)]
pub struct Interval {
    sleep: Pin<Box<Sleep>>,
    period: Duration,
    missed_tick_behavior: MissedTickBehavior,
}

impl Interval {
    /// Creates a new interval which will fire at `start` and then every `period` duration.
    pub fn new(
        handle: &TimersHandle,
        start: Instant,
        period: Duration,
        label: &'static str,
    ) -> Self {
        tracing::info!(?start, ?period, "Interval::new");
        Self {
            sleep: Box::pin(Sleep::new(handle, start, label)),
            period,
            missed_tick_behavior: MissedTickBehavior::Burst,
        }
    }

    pub async fn tick(&mut self) -> Instant {
        futures::future::poll_fn(move |cx| self.poll_tick(cx)).await
    }

    pub fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        let deadline = self.sleep.deadline();

        // Wait until the next tick
        ready!(self.sleep.as_mut().poll(cx));

        let now = Instant::now();

        // Calculate the next deadline
        // let new_deadline = deadline + self.period;
        let new_deadline = self
            .missed_tick_behavior
            .next_timeout(deadline, now, self.period);

        // Reset the timer
        // Note: will not be registered until the interval is polled again
        self.sleep.as_mut().reset(new_deadline);

        Poll::Ready(deadline)
    }
}

impl Stream for Interval {
    type Item = Instant;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_tick(cx).map(Some)
    }
}

#[cfg(test)]
mod test {

    use futures::StreamExt;

    use crate::time::{assert_dur, setup_timers};

    use super::*;

    fn assert_interval(
        start: Instant,
        stream: impl Stream<Item = Instant> + Unpin,
        expected: impl IntoIterator<Item = (Duration, Duration)>,
    ) {
        let mut expected_deadline = start;
        let mut last = start;

        for (i, (deadline, (expected_fixed, expected_wall))) in
            futures::executor::block_on_stream(stream)
                .zip(expected)
                .enumerate()
        {
            let elapsed = last.elapsed();
            last = Instant::now();

            eprintln!("[{i}] Took: {elapsed:?}");

            expected_deadline += expected_fixed;

            // What the deadline should have been
            // Compare the returned deadline to the expected one
            #[cfg(not(miri))]
            assert_dur(
                deadline.duration_since(start),
                expected_deadline.duration_since(start),
                "next returned deadline",
            );

            #[cfg(not(miri))]
            assert_dur(elapsed, expected_wall, "elapsed wall time");
        }
    }

    #[test]
    fn interval() {
        let (handle, j) = setup_timers();

        let now = Instant::now();

        let expected = [
            // First tick is immediate
            (Duration::ZERO, Duration::ZERO),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
        ];

        let interval = Interval::new(&handle, now, Duration::from_millis(100), "a");

        #[cfg(not(miri))]
        assert_interval(now, interval, expected);

        drop(handle);

        j.join().unwrap();
    }

    #[test]
    fn interval_burst() {
        let (handle, j) = setup_timers();

        let now = Instant::now();

        let delays = futures::stream::iter([
            Duration::ZERO,
            Duration::ZERO,
            Duration::from_millis(150),
            // Duration::from_millis(50),
            Duration::ZERO,
            Duration::from_millis(50),
            Duration::ZERO,
            Duration::from_millis(350),
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        ])
        .then(|d| Sleep::new(&handle, Instant::now() + d, "a"));

        let expected = [
            (Duration::ZERO, Duration::ZERO),
            // Normal tick
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(150)),
            // 50 ms behind
            (Duration::from_millis(100), Duration::from_millis(50)),
            // In phase
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(350)),
            // 250 ms behind
            (Duration::from_millis(100), Duration::ZERO),
            // 150 ms behind
            (Duration::from_millis(100), Duration::ZERO),
            // 50 ms behind
            (Duration::from_millis(100), Duration::from_millis(50)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
            (Duration::from_millis(100), Duration::from_millis(100)),
        ];

        let interval = Interval::new(&handle, now, Duration::from_millis(100), "b")
            .zip(delays)
            .map(|v| v.0);

        #[cfg(not(miri))]
        assert_interval(now, interval, expected);

        drop(handle);
        j.join().unwrap();
    }
}
