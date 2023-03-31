use std::{fmt::Display, num::NonZeroUsize};

use serde::{de::Visitor, Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    suffix: VersionSuffix,
}
impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, suffix: VersionSuffix) -> Self {
        Self {
            major,
            minor,
            patch,
            suffix,
        }
    }

    pub fn new_from_str(id: &str) -> Result<Self, VersionError> {
        if id.trim().is_empty() {
            return Err(VersionError::TooFewComponents);
        }

        let mut segments = id.split('.');
        let major = segments
            .next()
            .ok_or(VersionError::TooFewComponents)?
            .parse()?;
        let minor = segments.next().map(|s| s.parse()).transpose()?.unwrap_or(0);
        // We handle patch separately as it may have a suffix.
        let (patch, suffix) = if let Some(patch) = segments.next() {
            let (patch, suffix) = match patch.split_once('-') {
                Some((patch, suffix)) => (patch, Some(suffix)),
                None => (patch, None),
            };

            (
                patch.parse()?,
                VersionSuffix::new_from_str(suffix.unwrap_or_default())?,
            )
        } else {
            (0, VersionSuffix::Final)
        };

        if segments.next().is_some() {
            return Err(VersionError::TooManyComponents);
        }

        if [major, minor, patch].iter().all(|v| *v == 0) {
            return Err(VersionError::AllZero);
        }

        Ok(Self {
            major,
            minor,
            patch,
            suffix,
        })
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.suffix {
            VersionSuffix::Final => write!(f, "{}.{}.{}", self.major, self.minor, self.patch),
            suf => write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, suf),
        }
    }
}
impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(VersionVisitor)
    }
}

struct VersionVisitor;
impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a semantic dot-separated version with up to three components and an optional prefix",
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Version::new_from_str(v).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VersionSuffix {
    Other(String),
    Dev,
    Alpha(Option<NonZeroUsize>),
    Beta(Option<NonZeroUsize>),
    ReleaseCandidate(Option<NonZeroUsize>),
    Final,
}
impl VersionSuffix {
    const RELEASE_CANDIDATE: &str = "rc";
    const BETA: &str = "beta";
    const ALPHA: &str = "alpha";
    const DEV: &str = "dev";

    pub fn new_from_str(id: &str) -> Result<Self, VersionError> {
        if id.is_empty() {
            Ok(Self::Final)
        } else if let Some(version) = id.strip_prefix(Self::RELEASE_CANDIDATE) {
            Ok(Self::ReleaseCandidate(NonZeroUsize::new(version.parse()?)))
        } else if let Some(version) = id.strip_prefix(Self::BETA) {
            Ok(Self::Beta(NonZeroUsize::new(version.parse()?)))
        } else if let Some(version) = id.strip_prefix(Self::ALPHA) {
            Ok(Self::Alpha(NonZeroUsize::new(version.parse()?)))
        } else if id == Self::DEV {
            Ok(Self::Dev)
        } else {
            Ok(Self::Other(id.to_string()))
        }
    }
}
impl Display for VersionSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (prefix, suffix) = match self {
            VersionSuffix::Final => ("", None),
            VersionSuffix::ReleaseCandidate(v) => (Self::RELEASE_CANDIDATE, *v),
            VersionSuffix::Beta(v) => (Self::BETA, *v),
            VersionSuffix::Alpha(v) => (Self::ALPHA, *v),
            VersionSuffix::Dev => (Self::DEV, None),
            VersionSuffix::Other(v) => (v.as_str(), None),
        };

        write!(f, "{prefix}")?;
        if let Some(v) = suffix {
            write!(f, "{v}")?;
        }

        Ok(())
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum VersionError {
    #[error("invalid number in version segment")]
    InvalidNumber(#[from] std::num::ParseIntError),
    #[error("too few components in version (at least one required)")]
    TooFewComponents,
    #[error("too many components (at most three required)")]
    TooManyComponents,
    #[error("all components were zero")]
    AllZero,
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use crate::{Version, VersionError, VersionSuffix};

    #[test]
    fn can_parse_versions() {
        use Version as V;
        use VersionSuffix as VS;

        assert_eq!(V::new_from_str("1"), Ok(V::new(1, 0, 0, VS::Final)));
        assert_eq!(V::new_from_str("1.0"), Ok(V::new(1, 0, 0, VS::Final)));
        assert_eq!(V::new_from_str("1.0.0"), Ok(V::new(1, 0, 0, VS::Final)));
        assert_eq!(V::new_from_str("1.2.3"), Ok(V::new(1, 2, 3, VS::Final)));
        assert_eq!(
            V::new_from_str("1.2.3-rc1"),
            Ok(V::new(1, 2, 3, VS::ReleaseCandidate(NonZeroUsize::new(1))))
        );

        assert_eq!(V::new_from_str(""), Err(VersionError::TooFewComponents));
        assert_eq!(V::new_from_str("0.0.0"), Err(VersionError::AllZero));
        assert!(matches!(
            V::new_from_str("1.2.3patch"),
            Err(VersionError::InvalidNumber(_))
        ));
        assert_eq!(
            V::new_from_str("1.2.3.4"),
            Err(VersionError::TooManyComponents)
        );
    }

    #[test]
    fn can_roundtrip_serialize_versions() {
        use Version as V;
        use VersionSuffix as VS;

        let versions = [
            V::new(1, 0, 0, VS::Final),
            V::new(1, 0, 0, VS::Dev),
            V::new(1, 0, 0, VS::ReleaseCandidate(NonZeroUsize::new(1))),
            V::new(123, 456, 789, VS::ReleaseCandidate(NonZeroUsize::new(1))),
            V::new(123, 456, 789, VS::Final),
        ];

        for version in versions {
            assert_eq!(
                version,
                serde_json::from_str(&serde_json::to_string(&version).unwrap()).unwrap()
            );
        }
    }

    #[test]
    fn can_sort_versions() {
        use Version as V;
        use VersionSuffix as VS;

        let versions = [
            V::new(0, 0, 1, VS::Final),
            V::new(0, 1, 0, VS::Dev),
            V::new(0, 1, 0, VS::Final),
            V::new(0, 1, 1, VS::Final),
            V::new(0, 1, 12, VS::Final),
            V::new(1, 0, 0, VS::Other("pancakes".to_string())),
            V::new(1, 0, 0, VS::Dev),
            V::new(1, 0, 0, VS::Alpha(None)),
            V::new(1, 0, 0, VS::Alpha(NonZeroUsize::new(1))),
            V::new(1, 0, 0, VS::Beta(NonZeroUsize::new(1))),
            V::new(1, 0, 0, VS::ReleaseCandidate(None)),
            V::new(1, 0, 0, VS::ReleaseCandidate(NonZeroUsize::new(1))),
            V::new(1, 0, 0, VS::Final),
            V::new(123, 456, 789, VS::ReleaseCandidate(NonZeroUsize::new(1))),
            V::new(123, 456, 789, VS::Final),
        ];

        for [v1, v2] in versions.windows(2).map(|w| [&w[0], &w[1]]) {
            if *v1 >= *v2 {
                panic!("failed comparison: {v1} is not less than {v2}");
            }
        }
    }
}
