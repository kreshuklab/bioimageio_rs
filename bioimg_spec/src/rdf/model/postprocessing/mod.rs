use std::fmt::Display;

use crate::rdf::NonEmptyList;
use aspartial::AsPartial;

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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialPostprocessingDescr )]
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
#[aspartial(name = PartialScaleMeanVarianceDescr)]
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
