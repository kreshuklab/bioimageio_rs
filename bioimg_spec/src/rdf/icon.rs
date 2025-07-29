use std::{borrow::Borrow, str::FromStr};

use aspartial::AsPartial;

use super::file_reference::FileReference;

#[derive(thiserror::Error, Debug, Clone)]
pub enum IconParsingError {
    #[error("Not emoji: '{0}'")]
    NotEmoji(String),
    #[error("More than 2 graphemes: '{0}'")]
    MoreThanTwoGraphemes(String),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Icon {
    Emoji(EmojiIcon),
    FileRef(FileReference),
}

impl AsPartial for Icon{
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        match self {
            Self::Emoji(e) => e.into(),
            Self::FileRef(r) => r.to_partial(),
        }
    }
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
        let mut graphemes = ::unic::segment::Graphemes::new(value);

        fn grapheme_is_emoji(grapheme: &str) -> bool {
            let Some(first_char) = grapheme.chars().nth(0) else {
                return false
            };
            ::unic::emoji::char::is_emoji(first_char)
        }

        let Some(g) = graphemes.next() else {
            return Err(IconParsingError::NotEmoji(value.into()))
        };
        if !grapheme_is_emoji(g) {
            return Err(IconParsingError::NotEmoji(value.into()))
        }

        let Some(g) = graphemes.next() else {
            return Ok(Self(value.into()))
        };
        if !grapheme_is_emoji(g) {
            return Err(IconParsingError::NotEmoji(value.into()))
        }

        if graphemes.next().is_some(){
            return Err(IconParsingError::MoreThanTwoGraphemes(value.into()))
        }

        return Ok(Self(value.to_owned()));
    }
}

#[test]
fn test_emoji_icon_parsing(){
    // This is "woman" followed by "microscope", which collapses to "female scientist".
    // It might still show up as two glyphs for you if your font doesn't have the
    // "female scientist" glyph or if your editor does different text shaping shenanigans.
    let female_scientist = "👩‍🔬";
    let crab = "🦀";
    let test_str = format!("{female_scientist}{crab}");

    EmojiIcon::from_str(&test_str).expect("This should work as 2 graphemes");
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
