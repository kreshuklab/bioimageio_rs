use std::{borrow::Borrow, error::Error, fmt::Display, str::FromStr};

use crate::util::AsPartial;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, AsPartial)]
pub struct SlashlessString<T>(T);

// impl<T: Borrow<str>> AsPartial for SlashlessString<T>{
//     type Partial = String;
// }

impl<T: Display> Display for SlashlessString<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Borrow<str>> Borrow<str> for SlashlessString<T>{
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SlashlessStringError {
    #[error("{0}")]
    BadString(Box<dyn Error + 'static>),
    #[error("String has slashes: {0}")]
    ContainsSlashes(String),
}

impl<T> From<SlashlessString<T>> for String
where
    T: Borrow<str> + TryFrom<String>,
    <T as TryFrom<String>>::Error: Error + 'static
{
    fn from(value: SlashlessString<T>) -> Self {
        value.0.borrow().to_owned()
    }
}

impl<T> FromStr for SlashlessString<T>
where
    T: Borrow<str> + for <'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Error + 'static
{
    type Err = SlashlessStringError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl<T> TryFrom<&str> for SlashlessString<T>
where
    T: Borrow<str> + for <'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Error + 'static
{
    type Error = SlashlessStringError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let t = match T::try_from(value) {
            Ok(t) => t,
            Err(err) => return Err(SlashlessStringError::BadString(Box::new(err)))
        };
        if t.borrow().chars().find(|c| *c == '/' || *c == '\\').is_some() {
            return Err(SlashlessStringError::ContainsSlashes(t.borrow().into()));
        }
        Ok(Self(t))
    }
}


//FIXME: cane we live withhout this and just have From<&str> ?
impl<T> TryFrom<String> for SlashlessString<T>
where
    T: Borrow<str> + for <'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Error + 'static
{
    type Error = SlashlessStringError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
