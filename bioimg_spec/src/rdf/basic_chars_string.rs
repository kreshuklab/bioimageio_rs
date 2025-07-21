use std::{borrow::Borrow, error::Error, fmt::Display, str::FromStr};

use aspartial::AsPartial;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BasicCharsString<T>(T);

impl<T: Borrow<str>> AsPartial for BasicCharsString<T> {
    type Partial = String;
    fn to_partial(self) -> Self::Partial{
        self.0.borrow().to_owned()
    }
}

impl<T> BasicCharsString<T>{
    pub const ALLOWED_CHARS: [char;67] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
        'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
        'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
        'W', 'X', 'Y', 'Z',
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        '_', '-', ' ', '(', ')'
    ];
}

impl<T: Display> Display for BasicCharsString<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Borrow<str>> Borrow<str> for BasicCharsString<T>{
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BasicCharsStringError {
    #[error("{0}")]
    BadString(Box<dyn Error + 'static>),
    #[error("String '{original}' has forbidden char: {forbidden_char}")]
    ContainsForbiddenChar{original: String, forbidden_char: char},
}

impl<T> From<BasicCharsString<T>> for String
where
    T: Borrow<str> + TryFrom<String>,
    <T as TryFrom<String>>::Error: Error + 'static
{
    fn from(value: BasicCharsString<T>) -> Self {
        value.0.borrow().to_owned()
    }
}

impl<T> FromStr for BasicCharsString<T>
where
    T: Borrow<str> + for <'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Error + 'static
{
    type Err = BasicCharsStringError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl<T> TryFrom<&str> for BasicCharsString<T>
where
    T: Borrow<str> + for <'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Error + 'static
{
    type Error = BasicCharsStringError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let t = match T::try_from(value) {
            Ok(t) => t,
            Err(err) => return Err(BasicCharsStringError::BadString(Box::new(err)))
        };
        if let Some(forbidden_char) = t.borrow().chars().find(|c| !Self::ALLOWED_CHARS.contains(c)) {
            return Err(BasicCharsStringError::ContainsForbiddenChar{
                original: t.borrow().into(),
                forbidden_char 
            });
        }
        Ok(Self(t))
    }
}


//FIXME: cane we live withhout this and just have From<&str> ?
impl<T> TryFrom<String> for BasicCharsString<T>
where
    T: Borrow<str> + for <'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Error + 'static
{
    type Error = BasicCharsStringError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}


