#[derive(thiserror::Error, Debug)]
pub enum ClipDescrParsingError{
    #[error("Max '{max}' not greater than min '{min}'")]
    MaxNotGreaterThanMin{min: f32, max: f32},
    #[error("Undefined float values not allowed: min: '{min}', max: '{max}'")]
    UndefinedFloatValue{min: f32, max: f32},
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(try_from="ClipDescrMessage")]
pub struct ClipDescr {
    min: f32,
    max: f32,
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
pub struct ClipDescrMessage {
    pub min: f32,
    pub max: f32,
}
