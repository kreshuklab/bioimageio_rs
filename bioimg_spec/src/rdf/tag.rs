use std::{borrow::Borrow, str::FromStr};

use ::aspartial::AsPartial;

use super::{bounded_string::BoundedStringParsingError, BoundedString};


#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq, AsPartial)]
#[aspartial(newtype)]
pub struct Tag(BoundedString<1, 1024>);

impl Borrow<str> for Tag{
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl From<Tag> for String{
    fn from(value: Tag) -> Self {
        value.0.into()
    }
}

impl TryFrom<String> for Tag{
    type Error = BoundedStringParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(BoundedString::try_from(value)?))
    }
}

impl FromStr for Tag {
    type Err = BoundedStringParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(BoundedString::from_str(s)?))
    }
}
