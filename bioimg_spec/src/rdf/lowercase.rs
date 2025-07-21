use std::{borrow::Borrow, error::Error, fmt::Display, ops::Deref, str::FromStr};
use serde::{Deserialize, Serialize};

use aspartial::AsPartial;

#[derive(thiserror::Error, Debug)]
pub enum LowercaseParsingError {
    #[error("{source}")]
    BadString { source: Box<dyn Error + 'static> },
    #[error("Character at offset {idx} is not lowercase: {value}")]
    IsNotLowercase { value: String, idx: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, AsPartial)]
#[aspartial(newtype)]
pub struct Lowercase<T>(T);

impl<T: Into<String>> From<Lowercase<T>> for String{
    fn from(value: Lowercase<T>) -> Self {
        value.0.into()
    }
}

impl<T: Borrow<str>> Borrow<str> for Lowercase<T> {
    fn borrow(&self) -> &str {
        return self.0.borrow();
    }
}

impl<T: Display> Display for Lowercase<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Borrow<str>> Deref for Lowercase<T> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        return self.borrow();
    }
}

impl<T, E> FromStr for Lowercase<T>
where
    E: Error + 'static,
    T: for<'a> TryFrom<&'a str, Error = E>,
    T: Borrow<str>,
{
    type Err = LowercaseParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl<T, E> TryFrom<&str> for Lowercase<T>
where
    E: Error + 'static,
    T: for<'a> TryFrom<&'a str, Error = E>,
    T: Borrow<str>,
{
    type Error = LowercaseParsingError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let inner = match T::try_from(value) {
            Err(err) => return Err(LowercaseParsingError::BadString { source: Box::new(err) }),
            Ok(inner_val) => inner_val,
        };
        let inner_str: &str = inner.borrow();
        if let Some(uppercase_idx) = inner_str.chars().position(|c| c.is_uppercase()) {
            return Err(LowercaseParsingError::IsNotLowercase {
                value: inner_str.into(),
                idx: uppercase_idx,
            });
        }
        Ok(Self(inner))
    }
}

impl<T, E> TryFrom<String> for Lowercase<T>
where
    E: Error + 'static,
    T: TryFrom<String, Error = E>,
    T: Borrow<str>,
{
    type Error = LowercaseParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let inner = match T::try_from(value) {
            Err(err) => return Err(LowercaseParsingError::BadString { source: Box::new(err) }),
            Ok(inner_val) => inner_val,
        };
        let inner_str: &str = inner.borrow();
        if let Some(uppercase_idx) = inner_str.chars().position(|c| c.is_uppercase()) {
            return Err(LowercaseParsingError::IsNotLowercase {
                value: inner_str.into(),
                idx: uppercase_idx,
            });
        }
        Ok(Self(inner))
    }
}
