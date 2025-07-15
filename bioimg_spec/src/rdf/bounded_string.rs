use std::{borrow::Borrow, fmt::Display, ops::RangeInclusive, str::FromStr, sync::Arc};

use aspartial::AsPartial;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone)]
pub enum BoundedStringParsingError {
    BadLength { allowed: RangeInclusive<usize> },
}

impl std::fmt::Display for BoundedStringParsingError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::BadLength { allowed } => {
                write!(f, "Expected a string with at least {} and at most {} characters", allowed.start(), allowed.end())
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, Hash, PartialOrd, Ord)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct BoundedString<const MIN_CHARS: usize, const MAX_CHARS: usize>(Arc<str>);

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> AsPartial for BoundedString<MIN_CHARS, MAX_CHARS> {
    type Partial = String;
}

impl<const MAX_CHARS: usize> Default for BoundedString<0, MAX_CHARS> {
    fn default() -> Self {
        Self(Arc::from(""))
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> TryFrom<&str> for BoundedString<MIN_CHARS, MAX_CHARS> {
    type Error = BoundedStringParsingError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        const { assert!(MAX_CHARS >= MIN_CHARS) };
        let allowed = MIN_CHARS..=MAX_CHARS;
        if allowed.contains(&value.len()) {
            Ok(BoundedString(Arc::from(value)))
        } else {
            Err(BoundedStringParsingError::BadLength { allowed })
        }
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> FromStr for BoundedString<MIN_CHARS, MAX_CHARS> {
    type Err = BoundedStringParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TryFrom::<&str>::try_from(s)
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> TryFrom<String> for BoundedString<MIN_CHARS, MAX_CHARS> {
    type Error = BoundedStringParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> Display for BoundedString<MIN_CHARS, MAX_CHARS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> Borrow<str> for BoundedString<MIN_CHARS, MAX_CHARS> {
    fn borrow(&self) -> &str {
        return &self.0;
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> BoundedString<MIN_CHARS, MAX_CHARS> {
    pub fn as_str(&self) -> &str {
        return &self.0;
    }
}

impl<const MIN_CHARS: usize, const MAX_CHARS: usize> From<BoundedString<MIN_CHARS, MAX_CHARS>> for String {
    fn from(value: BoundedString<MIN_CHARS, MAX_CHARS>) -> Self {
        return value.0.as_ref().to_owned();
    }
}
