pub mod scale_linear;
pub mod binarize;
pub mod clip;
pub mod sigmoid;
pub mod zero_mean_unit_variance;
pub mod scale_range;
pub mod ensure_dtype;

use std::fmt::Display;
use std::str::FromStr;

use binarize::PartialBinarizeDescr;
use clip::PartialClipDescr;
use ensure_dtype::PartialEnsureDtype;
use scale_linear::PartialScaleLinearDescr;
use scale_range::PartialScaleRangeDescr;
use sigmoid::PartialSigmoid;
use zero_mean_unit_variance::{PartialFixedZmuv, PartialZmuv};

pub use self::scale_linear::{ScaleLinearDescr, SimpleScaleLinearDescr, ScaleLinearAlongAxisDescr};
pub use self::binarize::{BinarizeDescr, SimpleBinarizeDescr, BinarizeAlongAxisDescr};
pub use self::clip::ClipDescr;
pub use self::sigmoid::Sigmoid;
pub use self::scale_range::{ScaleRangeDescr, ScaleRangePercentile};
pub use self::ensure_dtype::EnsureDtype;
pub use self::zero_mean_unit_variance::Zmuv;
pub use self::zero_mean_unit_variance::{SimpleFixedZmuv, FixedZmuvAlongAxis, FixedZmuv};

use crate::util::{AsPartial, SingleOrMultiple};

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

#[derive(thiserror::Error, Debug, Clone)]
pub enum PreprocessingEpsilonParsingError{
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Preprocessing epsilon must be in open interval ]0, 0.1], found {0}")]
    OutOfRange(f32)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[derive(derive_more::Display, derive_more::Into)]
pub struct PreprocessingEpsilon(f32);

impl AsPartial for PreprocessingEpsilon {
    type Partial = f32;
}

impl Default for PreprocessingEpsilon{
    fn default() -> Self {
        Self(1e-6)
    }
}

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

impl FromStr for PreprocessingEpsilon{
    type Err = PreprocessingEpsilonParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(f32::from_str(s)?)
    }
}
// //////////////////

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "id", content = "kwargs")]
pub enum PreprocessingDescr {
    #[serde(rename = "binarize")]
    Binarize(BinarizeDescr),
    #[serde(rename = "clip")]
    Clip(ClipDescr),
    #[serde(rename = "ensure_dtype")]
    EnsureDtype(EnsureDtype),
    #[serde(rename = "scale_linear")]
    ScaleLinear(ScaleLinearDescr),
    #[serde(rename = "sigmoid")]
    Sigmoid(Sigmoid),
    #[serde(rename = "fixed_zero_mean_unit_variance")]
    FixedZeroMeanUnitVariance(FixedZmuv),
    #[serde(rename = "zero_mean_unit_variance")]
    ZeroMeanUnitVariance(Zmuv),
    #[serde(rename = "scale_range")]
    ScaleRange(ScaleRangeDescr),
}

impl AsPartial for PreprocessingDescr {
    type Partial = PartialPreprocessingDescr;
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(try_from="serde_json::Value")]
pub struct PartialPreprocessingDescr {
    pub binarize: Option<PartialBinarizeDescr>,
    pub clip: Option<PartialClipDescr>,
    pub ensure_dtype: Option<PartialEnsureDtype>,
    pub scale_linear: Option<PartialScaleLinearDescr>,
    pub sigmoid: Option<PartialSigmoid>,
    pub fixed_zero_mean_unit_variance: Option<PartialFixedZmuv>,
    pub zero_mean_unit_variance: Option<PartialZmuv>,
    pub scale_range: Option<PartialScaleRangeDescr>,
}

impl AsPartial for PartialPreprocessingDescr {
    type Partial = Self;
}

impl TryFrom<serde_json::Value> for PartialPreprocessingDescr {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {

        fn do_from_value(value: &serde_json::Value) -> Result<PartialPreprocessingDescr, serde_json::Error>{
            Ok(PartialPreprocessingDescr{
                binarize: serde_json::from_value(value.clone()).ok(),
                clip: serde_json::from_value(value.clone()).ok(),
                ensure_dtype: serde_json::from_value(value.clone()).ok(),
                scale_linear: serde_json::from_value(value.clone()).ok(),
                sigmoid: serde_json::from_value(value.clone()).ok(),
                fixed_zero_mean_unit_variance: serde_json::from_value(value.clone()).ok(),
                zero_mean_unit_variance: serde_json::from_value(value.clone()).ok(),
                scale_range: serde_json::from_value(value.clone()).ok(),
            })
        }

        let id = match value.get("id") {
            Some(serde_json::Value::String(s)) => Some(s),
            _ => None,
        };
        let kwargs = value.get("kwargs");

        let (id, value) = match (id, kwargs) {
            (None, None) => return do_from_value(&value),
            (None, Some(kwargs)) => return do_from_value(kwargs),
            (Some(id), None) => (id, &value),
            (Some(id), Some(kwargs)) => (id, kwargs),
        };

        let empty = Self{
            binarize: None,
            clip: None,
            ensure_dtype: None,
            scale_linear: None,
            sigmoid: None,
            fixed_zero_mean_unit_variance: None,
            zero_mean_unit_variance: None,
            scale_range: None,
        };

        Ok(match id.as_str() {
            "binarize" => Self{ binarize: serde_json::from_value(value.clone()).ok(), ..empty },
            "clip" => Self{ clip: serde_json::from_value(value.clone()).ok(), ..empty },
            "ensure_dtype" => Self{ ensure_dtype: serde_json::from_value(value.clone()).ok(), ..empty },
            "scale_linear" => Self{ scale_linear: serde_json::from_value(value.clone()).ok(), ..empty },
            "sigmoid" => Self{ sigmoid: serde_json::from_value(value.clone()).ok(), ..empty },
            "fixed_zero_mean_unit_variance" => Self{ fixed_zero_mean_unit_variance: serde_json::from_value(value.clone()).ok(), ..empty },
            "zero_mean_unit_variance" => Self{ zero_mean_unit_variance: serde_json::from_value(value.clone()).ok(), ..empty },
            "scale_range" => Self{ scale_range: serde_json::from_value(value.clone()).ok(), ..empty },
            _ => return do_from_value(&value),
        })
    }
}

impl Display for PreprocessingDescr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Binarize(prep) => prep.fmt(f),
            Self::Clip(prep) => prep.fmt(f),
            Self::EnsureDtype(prep) => prep.fmt(f),
            Self::ScaleLinear(prep) => prep.fmt(f),
            Self::Sigmoid(prep) => prep.fmt(f),
            Self::FixedZeroMeanUnitVariance(prep) => prep.fmt(f),
            Self::ZeroMeanUnitVariance(prep) => prep.fmt(f),
            Self::ScaleRange(prep) => prep.fmt(f),
        }
    }
}
