use std::time::Duration;

use itertools::{Itertools, PeekingNext};
use thiserror::Error;

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
            (TokenKind::Integral, None) => {
                num = Some(
                    head.parse()
                        .map_err(|_| DurationParseError::MalformedIntegral(head.to_string()))?,
                )
            }
            (TokenKind::Integral, Some(_)) => return Err(DurationParseError::DoubleIntegral),
            (TokenKind::Identifier, None) => {
                return Err(DurationParseError::MissingIntegral(head.to_string()))
            }
            (TokenKind::Identifier, Some(n)) => {
                let scale = DurationScale::parse(head)
                    .ok_or_else(|| DurationParseError::MalformedSuffix(head.to_string()))?;
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TokenKind {
    Integral,
    Identifier,
    WhiteSpace,
}

fn consume_integral(
    iter: &mut impl PeekingNext<Item = (usize, char)>,
) -> Option<(TokenKind, usize)> {
    iter.peeking_take_while(|(_, c)| c.is_ascii_digit() || *c == ',' || *c == '.')
        .last()
        .map(|(i, c)| (TokenKind::Integral, i + c.len_utf8()))
}

fn consume_ident(iter: &mut impl PeekingNext<Item = (usize, char)>) -> Option<(TokenKind, usize)> {
    iter.peeking_take_while(|(_, c)| c.is_alphabetic())
        .last()
        .map(|(i, c)| (TokenKind::Identifier, i + c.len_utf8()))
}

fn consume_whitespace(
    iter: &mut impl PeekingNext<Item = (usize, char)>,
) -> Option<(TokenKind, usize)> {
    iter.peeking_take_while(|(_, c)| c.is_whitespace() || matches!(*c, ',' | '.' | ':'))
        .last()
        .map(|(i, c)| (TokenKind::WhiteSpace, i + c.len_utf8()))
}

fn tok(s: &str) -> Option<(TokenKind, &str, &str)> {
    let mut iter = s.char_indices();
    let tok = consume_integral(&mut iter)
        .or_else(|| consume_ident(&mut iter))
        .or_else(|| consume_whitespace(&mut iter));

    if let Some((kind, tok)) = tok {
        let (head, tail) = s.split_at(tok);
        Some((kind, head, tail))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_duration() {
        let input = ["", "1s", "4m", "5m2s"];
        let output = input.into_iter().map(super::parse_duration).collect_vec();
        let expected = [
            Ok(Duration::ZERO),
            Ok(Duration::from_secs(1)),
            Ok(Duration::from_secs(240)),
            Ok(Duration::from_secs(302)),
        ];
        assert_eq!(output, expected);
    }
}
