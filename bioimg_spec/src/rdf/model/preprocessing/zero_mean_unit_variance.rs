use std::{fmt::Display, str::FromStr};

use aspartial::AsPartial;

use crate::rdf::{model::{axes::NonBatchAxisId, AxisId}, non_empty_list::NonEmptyList};
use super::PreprocessingEpsilon;

#[derive(thiserror::Error, Debug, Clone)]
pub enum ZmuvParsingError{
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Standard deviation must be greater or equal to 1e-6. Provided {0}")]
    BadStandardDeviation(f32),
    #[error("std and mean must have the same lengths")]
    MismatchedStdAndMean,
    #[error("Empty list")] //FIXME: this should never happen if using NonEmptyList
    EmptyList,
}

#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Debug, AsPartial)]
#[aspartial(newtype)]
pub struct ZmuvStdDeviation(f32);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialZmuv)]
pub struct Zmuv {
    /// The subset of axes to normalize jointly, i.e. axes to reduce to compute mean/std.
    /// For example to normalize 'batch', 'x' and 'y' jointly in a tensor ('batch', 'channel', 'y', 'x')
    /// resulting in a tensor of equal shape normalized per channel, specify `axes=('batch', 'x', 'y')`.
    /// To normalize each sample independently leave out the 'batch' axis.
    /// Default: Scale all axes jointly.
    pub axes: Option<NonEmptyList<AxisId>>,

    /// epsilon for numeric stability: `out = (tensor - mean) / (std + eps)`.
    #[serde(default)]
    pub eps: PreprocessingEpsilon,
}

impl Display for Zmuv{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mean=0 σ²=1 (ε={}", self.eps)?;
        if let Some(axes) = &self.axes{
            write!(f, ", axes={}", axes)?;
        }
        write!(f, ")")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, AsPartial)]
#[aspartial(name = PartialFixedZmuv)]
#[serde(untagged)]
pub enum FixedZmuv{
    Simple(SimpleFixedZmuv),
    AlongAxis(FixedZmuvAlongAxis)
}

impl Display for FixedZmuv{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Simple(prep) => prep.fmt(f),
            Self::AlongAxis(prep) => prep.fmt(f),
        }
    }
}

impl TryFrom<f32> for ZmuvStdDeviation{
    type Error = ZmuvParsingError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < 1e-6{
            return Err(ZmuvParsingError::BadStandardDeviation(value))
        }
        Ok(Self(value))
    }
}

impl Display for ZmuvStdDeviation{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ZmuvStdDeviation{
    type Err = ZmuvParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(f32::from_str(s)?)
    }
}

impl From<ZmuvStdDeviation> for f32{
    fn from(value: ZmuvStdDeviation) -> Self {
        value.0
    }
}

///Normalize with fixed, precomputed values for mean and variance.
///See `zero_mean_unit_variance` for data dependent normalization.
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, AsPartial)]
#[aspartial(name = PartialSimpleFixedZmuv)]
pub struct SimpleFixedZmuv{
    ///The mean value to normalize with.
    pub mean: f32,
    ///The standard deviation value to normalize with.
    pub std: ZmuvStdDeviation,
}

impl Display for SimpleFixedZmuv{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fixed mean=0 σ²=1 (from mean={}, σ={})", self.mean, self.std)
    }
}


// Normalize with fixed, precomputed values for mean and variance.
// See `zero_mean_unit_variance` for data dependent normalization.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(try_from = "FixedZmuvAlongAxisMsg")]
#[serde(into = "FixedZmuvAlongAxisMsg")]
pub struct FixedZmuvAlongAxis{
    pub mean_and_std: NonEmptyList<SimpleFixedZmuv>,
    pub axis: NonBatchAxisId,
}

impl AsPartial for FixedZmuvAlongAxis{
    type Partial = PartialFixedZmuvAlongAxisMsg;
    fn to_partial(self) -> Self::Partial {
        FixedZmuvAlongAxisMsg::from(self).to_partial()
    }
}

impl Display for FixedZmuvAlongAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fixed mean=0 σ²=1 along {}", self.axis)
    }
}

impl TryFrom<FixedZmuvAlongAxisMsg> for FixedZmuvAlongAxis{
    type Error = ZmuvParsingError;
    fn try_from(value: FixedZmuvAlongAxisMsg) -> Result<Self, Self::Error> {
        if value.mean.len() != value.std.len(){
            return Err(ZmuvParsingError::MismatchedStdAndMean)
        }
        return Ok(Self{
            mean_and_std: value.mean.into_inner().into_iter()
                .zip(value.std.into_inner().into_iter())
                .map(|(mean, std)| SimpleFixedZmuv{mean, std})
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| ZmuvParsingError::EmptyList)?,
            axis: value.axis,
        })
    }
}

impl From<FixedZmuvAlongAxis> for FixedZmuvAlongAxisMsg{
    fn from(value: FixedZmuvAlongAxis) -> Self {
        Self{
            mean: value.mean_and_std.map(|mean_and_std| mean_and_std.mean),
            std: value.mean_and_std.map(|mean_and_std| mean_and_std.std),
            axis: value.axis,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, AsPartial)]
#[aspartial(name = PartialFixedZmuvAlongAxisMsg)]
pub struct FixedZmuvAlongAxisMsg{
    /// The mean value(s) to normalize with.
    mean: NonEmptyList<f32>,
    /// The standard deviation value(s) to normalize with.
    /// Size must match `mean` values.
    std: NonEmptyList<ZmuvStdDeviation>,
    /// The axis of the mean/std values to normalize each entry along that dimension
    /// separately.
    axis: NonBatchAxisId,
}
