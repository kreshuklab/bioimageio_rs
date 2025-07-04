use std::{borrow::Borrow, error::Error, fmt::Display, ops::Deref, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::util::AsPartial;

const PYTHON_KEYWORDS: [&'static str; 35] = [
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else",
    "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
    "return", "try", "while", "with", "yield",
];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct Identifier(Arc<str>);

impl AsPartial for Identifier {
    type Partial = String;
}

impl Deref for Identifier{
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Identifier {
    pub fn appended_with(&self, suffix: &str) -> Self { //FIXME?
        return Self(Arc::from(format!("{}{suffix}", self.0).as_str()));
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        return self.0.borrow();
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IdentifierParsingError {
    #[error("Bad identifier: {source}")]
    BadString { source: Box<dyn Error + 'static> },
    #[error("Empty string can't be an identifier")]
    EmptyString,
    #[error("Expected first character to be alphabetic or _: '{value}'")]
    MustStartWithAlphabeticalOrUnderscore { value: String },
    #[error("Identifiers cannot contain whitespace: '{value}'")]
    ContainsbadCharacter { value: String, position: usize },
    #[error("Value '{value}' is a python keyword")]
    IsPythonKeyword { value: String },
}

impl FromStr for Identifier{
    type Err = IdentifierParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for Identifier{
    type Error = IdentifierParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Some(first_char) = value.chars().next() else {
            return Err(IdentifierParsingError::EmptyString);
        };
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(IdentifierParsingError::MustStartWithAlphabeticalOrUnderscore { value: value.into() });
        }
        for (idx, c) in value.char_indices() {
            if !c.is_alphanumeric() && c != '_' {
                return Err(IdentifierParsingError::ContainsbadCharacter {
                    value: value.into(),
                    position: idx,
                });
            }
        }
        if PYTHON_KEYWORDS.iter().copied().position(|kw| kw == value).is_some() {
            return Err(IdentifierParsingError::IsPythonKeyword { value: value.into() });
        }
        Ok(Self(Arc::from(value)))
    }
}

impl TryFrom<String> for Identifier{
    type Error = IdentifierParsingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        return value.0.as_ref().to_owned();
    }
}
