use std::{borrow::Borrow, str::FromStr};

use aspartial::AsPartial;

use super::file_reference::FileReference;

#[derive(thiserror::Error, Debug, Clone)]
pub enum IconParsingError {
    #[error("Not emoji: '{0}'")]
    NotEmoji(String),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Icon {
    Emoji(EmojiIcon),
    FileRef(FileReference),
}

impl AsPartial for Icon{
    type Partial = String;
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum EmojiParsingError {
    #[error("Bad string: {0}")]
    BadString(String),
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct EmojiIcon(String);

impl Borrow<str> for EmojiIcon{
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl FromStr for EmojiIcon{
    type Err = IconParsingError;
    //FIXME: check that characters/glyphs,graphemes/whatever are emoji
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !(1..=2).contains(&value.chars().count()) {
            return Err(IconParsingError::NotEmoji(value.to_owned()));
        }
        return Ok(Self(value.to_owned()));
    }
}

impl TryFrom<String> for EmojiIcon {
    type Error = IconParsingError;
    //FIXME: check that characters/glyphs,graphemes/whatever are emoji
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl TryFrom<String> for Icon {
    type Error = IconParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Icon::Emoji(EmojiIcon::try_from(value)?))
    }
}

impl From<EmojiIcon> for String {
    fn from(value: EmojiIcon) -> Self {
        return value.0;
    }
}
