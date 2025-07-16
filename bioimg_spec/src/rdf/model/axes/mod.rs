pub mod output_axes;
pub mod input_axes;


use std::{borrow::Borrow, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use aspartial::AsPartial;

use super::FixedAxisSize;
use super::axis_size::AnyAxisSize;
use crate::rdf::non_empty_list::NonEmptyList;
use crate::rdf::lowercase::{Lowercase, LowercaseParsingError};
use crate::rdf::literal::{LitStr, LiteralInt, StrMarker};
use crate::rdf::identifier::Identifier;
use crate::rdf::bounded_string::BoundedString;

pub type AxisId = Lowercase<BoundedString<1, 16>>;
pub type AxisDescription = BoundedString<0, 128>;

#[derive(thiserror::Error, Debug)]
pub enum AxisIdParsingError{
    #[error(transparent)]
    LowercaseParsingError(#[from] LowercaseParsingError),
    #[error("Axis can't be 'batch'")]
    CantBeBatch,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct NonBatchAxisId(AxisId);

impl AsPartial for NonBatchAxisId {
    type Partial = String;
}

impl Display for NonBatchAxisId{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<AxisId> for NonBatchAxisId{
    fn borrow(&self) -> &AxisId {
        &self.0
    }
}

impl Borrow<str> for NonBatchAxisId{
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl FromStr for NonBatchAxisId{
    type Err = AxisIdParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let axis_id = AxisId::from_str(s)?;
        Self::try_from(axis_id)
    }
}

impl TryFrom<AxisId> for NonBatchAxisId{
    type Error = AxisIdParsingError;
    fn try_from(value: AxisId) -> Result<Self, Self::Error> {
        let raw_axis_id: &'_ str = value.borrow();
        if raw_axis_id == "batch"{
            Err(AxisIdParsingError::CantBeBatch)
        }else{
            Ok(Self(value))
        }
    }
}

impl TryFrom<String> for NonBatchAxisId{
    type Error = AxisIdParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let axisid = AxisId::try_from(value)?;
        Ok(Self::try_from(axisid)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct AxisScale(f32);

impl AsPartial for AxisScale {
    type Partial = f32;
}

impl Display for AxisScale{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AxisScale{
    type Err = std::num::ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(f32::from_str(s)?))
    }
}

impl From<AxisScale> for f32{
    fn from(value: AxisScale) -> Self {
        value.0
    }
}

impl Default for AxisScale {
    fn default() -> Self {
        Self(1.0)
    }
}

//FIXME: why isn't Halo just a NonNegativeU64?
#[derive(thiserror::Error, Debug, Clone)]
pub enum HaloParsingError {
    #[error("Halo must be a positive integer, found {found}")]
    MustBePositive { found: u64 },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Halo(u64);

impl AsPartial for Halo {
    type Partial = u64;
}

impl From<Halo> for u64{
    fn from(value: Halo) -> Self {
        value.0
    }
}

impl TryFrom<u64> for Halo {
    type Error = HaloParsingError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(HaloParsingError::MustBePositive { found: value });
        }
        Ok(Self(value))
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Batch;
impl StrMarker for Batch { const NAME: &'static str = "batch"; }

#[derive(Default, Clone, Copy, Debug)]
pub struct Index;
impl StrMarker for Index { const NAME: &'static str = "index"; }

#[derive(Default, Clone, Copy, Debug)]
pub struct Channel;
impl StrMarker for Channel { const NAME: &'static str = "channel"; }

#[derive(Default, Clone, Copy, Debug)]
pub struct Space;
impl StrMarker for Space { const NAME: &'static str = "space"; }

#[derive(Default, Clone, Copy, Debug)]
pub struct Time;
impl StrMarker for Time { const NAME: &'static str = "time"; }


#[derive(
    serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default,
    strum::VariantArray, strum::AsRefStr, strum::Display,
)]
pub enum AxisType {
    #[serde(rename = "batch")]
    Batch,
    #[serde(rename = "channel")]
    Channel,
    #[serde(rename = "index")]
    Index,
    #[serde(rename = "time")]
    Time,
    #[default]
    #[serde(rename = "space")]
    Space,
}

#[derive(thiserror::Error, PartialEq, Clone, Debug)]
pub enum AxisScaleParsingError {
    #[error("Axis scale is less than 0.0: {0}")]
    LessThanZero(f32),
}

impl TryFrom<f32> for AxisScale {
    type Error = AxisScaleParsingError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value > 0.0 {
            Ok(Self(value))
        } else {
            Err(AxisScaleParsingError::LessThanZero(value))
        }
    }
}

// ///////////////////////

#[derive(Serialize, Deserialize, Debug, Clone, Default, AsPartial)]
#[aspartial(name = PartialBatchAxis)]
pub struct BatchAxis {
    #[serde(default)]
    pub id: LitStr<Batch>,
    #[serde(default)]
    pub description: AxisDescription,
    #[serde(default)]
    pub size: Option<LiteralInt<1>>,
}

impl Display for BatchAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Batch")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialChannelAxis)]
pub struct ChannelAxis {
    #[serde(default)]
    pub id: LitStr<Channel>,
    #[serde(default)]
    pub description: AxisDescription,
    // pub size: FixedAxisSize,
    pub channel_names: NonEmptyList<Identifier>,
    // #[serde(default)]
    // pub channel_names: ChannelNames, // FIXME: do we need to handle "#channel_names" ?
}

impl Display for ChannelAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Channels {}", self.channel_names)
    }
}

impl ChannelAxis{
    pub fn size(&self) -> FixedAxisSize{
        self.channel_names.len()
    }
}

impl ChannelAxis {
    pub fn is_compatible_with_extent(&self, extent: usize) -> bool {
        let len: usize = self.channel_names.len().into();
        len == extent
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialIndexAxis)]
pub struct IndexAxis {
    #[serde(default)]
    pub id: LitStr<Index>,
    #[serde(default)]
    pub description: AxisDescription,
    pub size: AnyAxisSize,
}

impl Display for IndexAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index") //FIXME: maybe include size?
    }
}

fn _default_time_axis_id() -> AxisId {
    String::from("time").try_into().unwrap()
}
fn _default_space_axis_id() -> AxisId {
    String::from("x").try_into().unwrap()
}
fn _default_axis_scale() -> f32 {
    1.0
}

#[derive(thiserror::Error, Debug)]
pub enum AxisGroupValidationError {
    #[error("Tensor axes descriptions cannot be empty")]
    Empty,
    #[error("Repeated Axis type: {0:?}")] //FIXME: don't use debug repr
    RepeatedAxisType(AxisType),
}

macro_rules!  impl_axis_group{($inout:ident) => { paste::paste!{
    impl std::ops::Deref for [<$inout AxisGroup>] {
        type Target = [ [<$inout Axis>] ];

        fn deref(&self) -> &Self::Target {
            return &self.0;
        }
    }

    impl TryFrom<Vec< [<$inout Axis>] >> for [<$inout AxisGroup>] {
        type Error = crate::rdf::model::axes::AxisGroupValidationError;
        fn try_from(value: Vec< [<$inout Axis>] >) -> Result<Self, Self::Error> {
            if value.len() == 0 {
                return Err(crate::rdf::model::axes::AxisGroupValidationError::Empty);
            }
            let mut axis_types = std::collections::HashSet::<AxisType>::with_capacity(5); //FIXME: 5?
            for val in value.iter() {
                if ! matches!(val.axis_type(), AxisType::Space) && !axis_types.insert(val.axis_type()) {
                    return Err(crate::rdf::model::axes::AxisGroupValidationError::RepeatedAxisType(val.axis_type()));
                }
            }
            //FIXME: there's probably other invalid stuff like only Batch or only Index
            return Ok(Self(value));
        }
    }
}};}

pub(crate) use impl_axis_group;
