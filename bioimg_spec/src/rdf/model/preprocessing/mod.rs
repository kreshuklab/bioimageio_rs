pub mod scale_linear;
pub mod binarize;
pub mod clip;
pub mod sigmoid;
pub mod zero_mean_unit_variance;
pub mod scale_range;

pub use self::scale_linear::ScaleLinearDescr;
pub use self::binarize::{BinarizeDescr, SimpleBinarizeDescr, BinarizeAlongAxisDescr};
pub use self::clip::ClipDescr;
pub use self::sigmoid::Sigmoid;
pub use self::zero_mean_unit_variance::ZeroMeanUnitVariance;
pub use self::scale_range::{ScaleRangeDescr, ScaleRangePercentile};

use crate::util::SingleOrMultiple;

// //////////////

fn _default_to_0f32() -> f32{
    0.0
}

fn _default_to_100f32() -> f32{
    100.0
}

fn _default_to_1() -> f32{
    1.0
}

fn _default_to_single_1() -> SingleOrMultiple<f32>{
    SingleOrMultiple::Single(1.0)
}

fn _default_to_single_0() -> SingleOrMultiple<f32>{
    SingleOrMultiple::Single(0.0)
}

#[derive(thiserror::Error, Debug)]
pub enum PreprocessingEpsilonParsingError{
    #[error("Preprocessing epsilon must be in open interval ]0, 0.1], found {0}")]
    OutOfRange(f32)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub struct PreprocessingEpsilon(f32);

impl TryFrom<f32> for PreprocessingEpsilon{
    type Error = PreprocessingEpsilonParsingError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value > 0.0 && value <= 0.1{
            Ok(Self(value))
        }else{
            Err(PreprocessingEpsilonParsingError::OutOfRange(value))
        }
    }
}

// //////////////////

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "name", content = "kwargs")]
pub enum PreprocessingDescr {
    #[serde(rename = "binarize")]
    Binarize(BinarizeDescr),
    #[serde(rename = "clip")]
    Clip(ClipDescr),
    #[serde(rename = "scale_linear")]
    ScaleLinear(ScaleLinearDescr),
    #[serde(rename = "sigmoid")]
    Sigmoid(Sigmoid),
    #[serde(rename = "zero_mean_unit_variance")]
    ZeroMeanUnitVariance(ZeroMeanUnitVariance),
    #[serde(rename = "scale_range")]
    ScaleRange(ScaleRangeDescr),
}
