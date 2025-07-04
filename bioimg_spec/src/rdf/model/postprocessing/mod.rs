use std::fmt::Display;

use crate::rdf::NonEmptyList;
use crate::util::AsPartial;

use super::preprocessing::clip::PartialClipDescr;
use super::preprocessing::ensure_dtype::PartialEnsureDtype;
use super::preprocessing::scale_linear::PartialScaleLinearDescr;
use super::preprocessing::scale_range::PartialScaleRangeDescr;
use super::preprocessing::sigmoid::PartialSigmoid;
use super::preprocessing::zero_mean_unit_variance::{PartialFixedZmuv, PartialZmuv};
use super::{AxisId, TensorId};
use super::preprocessing::{BinarizeDescr, ClipDescr, EnsureDtype, FixedZmuv, PreprocessingEpsilon, ScaleLinearDescr, ScaleRangeDescr, Sigmoid, Zmuv};
use super::preprocessing::binarize::PartialBinarizeDescr;


// Note: be careful when editing this, as the partial version has to match
// precisely
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "id", content = "kwargs")]
pub enum PostprocessingDescr {
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
    #[serde(rename = "scale_mean_variance")]
    ScaleMeanVarianceDescr(ScaleMeanVarianceDescr),
}

impl AsPartial for PostprocessingDescr {
    type Partial = PartialPostprocessingDescr;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PartialPostprocessingDescr {
    pub binarize: Option<PartialBinarizeDescr>,
    pub clip: Option<PartialClipDescr>,
    pub ensure_dtype: Option<PartialEnsureDtype>,
    pub scale_linear: Option<PartialScaleLinearDescr>,
    pub sigmoid: Option<PartialSigmoid>,
    pub fixed_zero_mean_unit_variance: Option<PartialFixedZmuv>,
    pub zero_mean_unit_variance: Option<PartialZmuv>,
    pub scale_range: Option<PartialScaleRangeDescr>,
    pub scale_mean_variance_descr: Option<PartialScaleMeanVarianceDescr>,
}

impl AsPartial for PartialPostprocessingDescr {
    type Partial = Self;
}

impl TryFrom<serde_json::Value> for PartialPostprocessingDescr {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {

        fn do_from_value(value: &serde_json::Value) -> Result<PartialPostprocessingDescr, serde_json::Error>{
            Ok(PartialPostprocessingDescr{
                binarize: serde_json::from_value(value.clone()).ok(),
                clip: serde_json::from_value(value.clone()).ok(),
                ensure_dtype: serde_json::from_value(value.clone()).ok(),
                scale_linear: serde_json::from_value(value.clone()).ok(),
                sigmoid: serde_json::from_value(value.clone()).ok(),
                fixed_zero_mean_unit_variance: serde_json::from_value(value.clone()).ok(),
                zero_mean_unit_variance: serde_json::from_value(value.clone()).ok(),
                scale_range: serde_json::from_value(value.clone()).ok(),
                scale_mean_variance_descr: serde_json::from_value(value.clone()).ok(),
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
            scale_mean_variance_descr: None,
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
            "scale_mean_variance" => Self { scale_mean_variance_descr: serde_json::from_value(value.clone()).ok(), ..empty },
            _ => return do_from_value(&value),
        })
    }
}

impl Display for PostprocessingDescr{
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
            Self::ScaleMeanVarianceDescr(prep) => prep.fmt(f),
        }
    }
}
/// Scale a tensor's data distribution to match another tensor's mean/std.
/// `out  = (tensor - mean) / (std + eps) * (ref_std + eps) + ref_mean.`
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
pub struct ScaleMeanVarianceDescr{
    /// Name of tensor to match.
    pub reference_tensor: TensorId,

    /// The subset of axes to normalize jointly, i.e. axes to reduce to compute mean/std.
    /// For example to normalize 'batch', 'x' and 'y' jointly in a tensor ('batch', 'channel', 'y', 'x')
    /// resulting in a tensor of equal shape normalized per channel, specify `axes=('batch', 'x', 'y')`.
    /// To normalize samples independently, leave out the 'batch' axis.
    /// default: Scale all axes jointly.
    pub axes: Option<NonEmptyList<AxisId>>,

    /// Epsilon for numeric stability:
    /// `out  = (tensor - mean) / (std + eps) * (ref_std + eps) + ref_mean.`
    #[serde(default)]
    pub eps: PreprocessingEpsilon,

}

impl Display for ScaleMeanVarianceDescr{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scale Mean Variance(ε={}, ref='{}'", self.eps, self.reference_tensor)?;
        if let Some(axes) = &self.axes{
            write!(f, ", axes={axes}")?;
        }
        write!(f, ")")
    }
}
