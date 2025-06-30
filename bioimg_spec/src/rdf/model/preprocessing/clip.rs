use std::fmt::Display;

use crate::util::AsPartial;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ClipDescrParsingError{
    #[error("Max '{max}' not greater than min '{min}'")]
    MaxNotGreaterThanMin{min: f32, max: f32},
    #[error("Undefined float values not allowed: min: '{min}', max: '{max}'")]
    UndefinedFloatValue{min: f32, max: f32},
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
#[serde(try_from="ClipDescrMessage")]
pub struct ClipDescr {
    min: f32,
    max: f32,
}

impl Display for ClipDescr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clip (min: {}, max: {})", self.min, self.max)
    }
}

impl ClipDescr {
    pub fn min(&self) ->  f32{
        self.min
    }
    pub fn max(&self) ->  f32{
        self.max
    }
    pub fn try_from_min_max(min: f32, max: f32) -> Result<Self, ClipDescrParsingError>{
        Self::try_from(ClipDescrMessage{min, max})
    }
}


impl TryFrom<ClipDescrMessage> for ClipDescr{
    type Error = ClipDescrParsingError;
    fn try_from(value: ClipDescrMessage) -> Result<Self, Self::Error> {
        if value.max.is_nan() || value.min.is_nan(){
            return Err(ClipDescrParsingError::UndefinedFloatValue { min: value.min, max: value.max })
        }
        if value.min >= value.max{
            return Err(ClipDescrParsingError::MaxNotGreaterThanMin { min: value.min, max: value.max })
        }
        Ok(Self{max: value.max, min: value.min})
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClipDescrMessage {
    pub min: f32,
    pub max: f32,
}
