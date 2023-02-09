//! Provides time related functionality like Clocks and TimeInfo. Also extends Duration for easier
//! construction like 5.secs().
use std::time::{Duration, Instant, SystemTime};

use itertools::{Itertools, PeekingNext};
use thiserror::Error;

/// Measures high precision time
#[derive(Debug, Clone)]
pub struct Clock {
    start: Instant,
}

impl Clock {
    // Creates and starts a new clock
    pub fn new() -> Self {
        Clock { start: Instant::now() }
    }

    // Returns the elapsed time
    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.start
    }

    // Resets the clock and returns the elapsed time
    pub fn reset(&mut self) -> Duration {
        let elapsed = self.elapsed();

        self.start = Instant::now();
        elapsed
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeout {
    timeout: Duration,
    start: Instant,
}

impl Default for Timeout {
    fn default() -> Self {
        Self { timeout: Default::default(), start: Instant::now() }
    }
}

impl Timeout {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout, start: Instant::now() }
    }

    pub fn empty() -> Self {
        Self { timeout: Duration::ZERO, start: Instant::now() }
    }

    pub fn set_duration(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.start = Instant::now();
        self
    }

    pub fn is_finished(&self) -> bool {
        self.start.elapsed() >= self.timeout
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn remaining(&self) -> Duration {
        self.timeout - self.start.elapsed()
    }

    pub fn duration(&self) -> Duration {
        self.timeout
    }
}

/// Allows shorter function names to convert duration into intergral types
pub trait FromDuration {
    fn secs(&self) -> f32;
    fn ms(&self) -> u128;
    fn us(&self) -> u128;
    fn ns(&self) -> u128;
}

impl FromDuration for Duration {
    fn secs(&self) -> f32 {
        self.as_secs_f32()
    }

    fn ms(&self) -> u128 {
        self.as_millis()
    }

    fn us(&self) -> u128 {
        self.as_micros()
    }

    fn ns(&self) -> u128 {
        self.as_nanos()
    }
}

/// Trait that allows easier construction of durations
pub trait IntoDuration {
    fn secs(&self) -> Duration;
    fn ms(&self) -> Duration;
    fn us(&self) -> Duration;
    fn ns(&self) -> Duration;
}

impl IntoDuration for u64 {
    fn secs(&self) -> Duration {
        Duration::from_secs(*self)
    }

    fn ms(&self) -> Duration {
        Duration::from_millis(*self)
    }

    fn us(&self) -> Duration {
        Duration::from_micros(*self)
    }

    fn ns(&self) -> Duration {
        Duration::from_nanos(*self)
    }
}

impl IntoDuration for f32 {
    fn secs(&self) -> Duration {
        Duration::from_secs_f32(*self)
    }

    fn ms(&self) -> Duration {
        Duration::from_secs_f64(*self as f64 / 1000.0)
    }

    fn us(&self) -> Duration {
        Duration::from_secs_f64(*self as f64 / 1_000_000.0)
    }

    fn ns(&self) -> Duration {
        Duration::from_secs_f64(*self as f64 / 1_000_000_000.0)
    }
}

/// Times the execution time of a scope and executes the provided function with
/// the results
pub struct TimedScope<F: FnOnce(Duration)> {
    func: Option<F>,
    clock: Clock,
}

impl<F: FnOnce(Duration)> TimedScope<F> {
    pub fn new(func: F) -> Self {
        TimedScope { func: Some(func), clock: Clock::new() }
    }
}

impl<F: FnOnce(Duration)> Drop for TimedScope<F> {
    fn drop(&mut self) {
        let elapsed = self.clock.elapsed();
        if let Some(f) = self.func.take() {
            f(elapsed)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TokenKind {
    Integral,
    Identifier,
    WhiteSpace,
}

fn consume_integral(iter: &mut impl PeekingNext<Item = (usize, char)>) -> Option<(TokenKind, usize)> {
    iter.peeking_take_while(|(_, c)| c.is_ascii_digit() || *c == ',' || *c == '.')
        .last()
        .map(|(i, c)| (TokenKind::Integral, i + c.len_utf8()))
}

fn consume_ident(iter: &mut impl PeekingNext<Item = (usize, char)>) -> Option<(TokenKind, usize)> {
    iter.peeking_take_while(|(_, c)| c.is_alphabetic()).last().map(|(i, c)| (TokenKind::Identifier, i + c.len_utf8()))
}

fn consume_whitespace(iter: &mut impl PeekingNext<Item = (usize, char)>) -> Option<(TokenKind, usize)> {
    iter.peeking_take_while(|(_, c)| c.is_whitespace() || matches!(*c, ',' | '.' | ':'))
        .last()
        .map(|(i, c)| (TokenKind::WhiteSpace, i + c.len_utf8()))
}

fn tok(s: &str) -> Option<(TokenKind, &str, &str)> {
    let mut iter = s.char_indices();
    let tok = consume_integral(&mut iter).or_else(|| consume_ident(&mut iter)).or_else(|| consume_whitespace(&mut iter));

    if let Some((kind, tok)) = tok {
        let (head, tail) = s.split_at(tok);
        Some((kind, head, tail))
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub enum DurationParseError {
    #[error("Missing number for suffix {0}")]
    MissingIntegral(String),
    #[error("Duplicate suffix")]
    DoubleSuffix,
    #[error("Duplicate number without identifier")]
    DoubleIntegral,
    #[error("Malformed integral")]
    MalformedIntegral(String),
    #[error("Malformed suffix")]
    MalformedSuffix(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DurationScale {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
}

impl DurationScale {
    pub fn to_duration(&self, time: f64) -> Duration {
        let scale = match self {
            DurationScale::Milliseconds => 1e-3,
            DurationScale::Seconds => 1.0,
            DurationScale::Minutes => 60.0,
            DurationScale::Hours => 3600.0,
        };

        Duration::from_secs_f64(time * scale)
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ms" | "millis" | "millisecond" | "milliseconds" => Some(Self::Milliseconds),
            "s" | "sec" | "second" | "seconds" => Some(Self::Seconds),
            "m" | "min" | "minute" | "minutes" => Some(Self::Minutes),
            "h" | "hour" | "hours" => Some(Self::Hours),
            _ => None,
        }
    }
}
/// Parses a duration in the format of `45` or `45s 1m`. Is overly relaxed and
/// will ignore spaces and mispellings.
pub fn parse_duration(mut s: &str) -> Result<Duration, DurationParseError> {
    let mut num: Option<f64> = None;

    let mut dur = Duration::ZERO;
    while let Some((kind, head, tail)) = tok(s) {
        match (kind, num) {
            (TokenKind::Integral, None) => num = Some(head.parse().map_err(|_| DurationParseError::MalformedIntegral(head.to_string()))?),
            (TokenKind::Integral, Some(_)) => return Err(DurationParseError::DoubleIntegral),
            (TokenKind::Identifier, None) => return Err(DurationParseError::MissingIntegral(head.to_string())),
            (TokenKind::Identifier, Some(n)) => {
                let scale = DurationScale::parse(head).ok_or_else(|| DurationParseError::MalformedSuffix(head.to_string()))?;
                dur += scale.to_duration(n);
                num = None;
            }
            (TokenKind::WhiteSpace, _) => {}
        }
        // Consume
        s = tail;
    }

    // Anything without a suffix is considered as seconds
    if let Some(num) = num {
        dur += DurationScale::Seconds.to_duration(num);
    }

    Ok(dur)
}

pub fn from_now(time: SystemTime) -> Option<String> {
    let duration = SystemTime::now().duration_since(time).ok()?;
    Some(format!("{} ago", pretty_duration(duration)))
}
pub fn pretty_duration(duration: Duration) -> String {
    let mut secs = duration.as_secs();
    if secs == 0 {
        return format!("{} ms", duration.as_millis());
    }

    let years = secs / (86400.0 * 365.2422) as u64;
    secs %= (86400.0 * 365.2422) as u64;

    let days = secs / 86400;
    secs %= 86400;

    let hours = secs / 3600;
    secs %= 3600;

    let minutes = secs / 60;
    secs %= 60;

    let mut res = Vec::new();

    if years > 0 {
        res.push(format!("{years} years"))
    }

    if days > 0 {
        res.push(format!("{days} days"))
    }

    if years == 0 && days == 0 {
        if hours > 0 {
            res.push(format!("{hours} hours"))
        }

        if minutes > 0 {
            res.push(format!("{minutes} minutes"))
        }

        if secs > 0 {
            res.push(format!("{secs} seconds"))
        }
    }

    res.join(" ")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_duration() {
        let input = ["", "1s", "4m", "5m2s"];
        let output = input.into_iter().map(super::parse_duration).collect_vec();
        let expected = [Ok(Duration::ZERO), Ok(Duration::from_secs(1)), Ok(Duration::from_secs(240)), Ok(Duration::from_secs(302))];
        assert_eq!(output, expected);
    }
}
