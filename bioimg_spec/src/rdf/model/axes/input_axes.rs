use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{rdf::model::{AnyAxisSize, SpaceUnit, TimeUnit}, util::AsPartial};

use super::{
    AxisDescription, AxisId, AxisScale, AxisType, BatchAxis, ChannelAxis, IndexAxis,
    _default_space_axis_id, _default_time_axis_id, impl_axis_group, PartialBatchAxis, PartialChannelAxis, PartialIndexAxis
};

#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
pub struct TimeInputAxis {
    #[serde(default = "_default_time_axis_id")]
    pub id: AxisId,
    #[serde(default)]
    pub description: AxisDescription,
    #[serde(default)]
    pub unit: Option<TimeUnit>,
    #[serde(default)]
    pub scale: AxisScale,
    pub size: AnyAxisSize,
}

impl Display for TimeInputAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Time: {}", self.id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
pub struct SpaceInputAxis {
    #[serde(default = "_default_space_axis_id")]
    pub id: AxisId,
    #[serde(default)]
    pub description: AxisDescription,
    #[serde(default)]
    pub unit: Option<SpaceUnit>,
    #[serde(default)]
    pub scale: AxisScale,
    pub size: AnyAxisSize,
}

impl Display for SpaceInputAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Space: {}", self.id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum InputAxis {
    #[serde(rename = "batch")]
    Batch(BatchAxis),
    #[serde(rename = "channel")]
    Channel(ChannelAxis),
    #[serde(rename = "index")]
    Index(IndexAxis),
    #[serde(rename = "time")]
    Time(TimeInputAxis),
    #[serde(rename = "space")]
    Space(SpaceInputAxis),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "type")]
#[serde(try_from="serde_json::Value")]
pub struct PartialInputAxis {
    pub batch: Option<PartialBatchAxis>,
    pub channel: Option<PartialChannelAxis>,
    pub index: Option<PartialIndexAxis>,
    pub time: Option<PartialTimeInputAxis>,
    pub space: Option<PartialSpaceInputAxis>,
}

impl TryFrom<serde_json::Value> for PartialInputAxis {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {

        fn do_from_value(value: &serde_json::Value) -> Result<PartialInputAxis, serde_json::Error>{
            Ok(PartialInputAxis {
                batch: serde_json::from_value(value.clone()).ok(),
                channel: serde_json::from_value(value.clone()).ok(),
                index: serde_json::from_value(value.clone()).ok(),
                time: serde_json::from_value(value.clone()).ok(),
                space: serde_json::from_value(value.clone()).ok(),
            })
        }

        let id = match value.get("type") {
            Some(serde_json::Value::String(s)) => s,
            _ => return do_from_value(&value),
        };

        let empty = Self {
            batch: None,
            channel: None,
            index: None,
            time: None,
            space: None,
        };

        Ok(match id.as_str() {
            "batch" => Self{ batch: serde_json::from_value(value.clone()).ok(), ..empty },
            "channel" => Self{ channel: serde_json::from_value(value.clone()).ok(), ..empty },
            "index" => Self{ index: serde_json::from_value(value.clone()).ok(), ..empty },
            "time" => Self{ time: serde_json::from_value(value.clone()).ok(), ..empty },
            "space" => Self{ space: serde_json::from_value(value.clone()).ok(), ..empty },
            _ => return do_from_value(&value),
        })
    }
}

impl Display for InputAxis{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Batch(axis) => axis.fmt(f),
            Self::Channel(axis) => axis.fmt(f),
            Self::Index(axis) => axis.fmt(f),
            Self::Time(axis) => axis.fmt(f),
            Self::Space(axis) => axis.fmt(f),
        }
    }
}

impl From<BatchAxis> for InputAxis{
    fn from(value: BatchAxis) -> Self {
        InputAxis::Batch(value)
    }
}
impl From<ChannelAxis> for InputAxis{
    fn from(value: ChannelAxis) -> Self {
        InputAxis::Channel(value)
    }
}
impl From<IndexAxis> for InputAxis{
    fn from(value: IndexAxis) -> Self {
        InputAxis::Index(value)
    }
}
impl From<TimeInputAxis> for InputAxis{
    fn from(value: TimeInputAxis) -> Self {
        InputAxis::Time(value)
    }
}
impl From<SpaceInputAxis> for InputAxis{
    fn from(value: SpaceInputAxis) -> Self {
        InputAxis::Space(value)
    }
}

impl InputAxis{
    pub fn axis_type(&self) -> AxisType {
        match self {
            Self::Batch(_) => AxisType::Batch,
            Self::Channel(_) => AxisType::Channel,
            Self::Index(_) => AxisType::Index,
            Self::Time(_) => AxisType::Time,
            Self::Space(_) => AxisType::Space,
       }
    }

    pub fn id(&self) -> AxisId {
        match self {
            Self::Batch(axis) => AxisId::try_from(axis.id.as_str().to_owned()).unwrap(),
            Self::Channel(axis) => AxisId::try_from(axis.id.as_str().to_owned()).unwrap(),
            Self::Index(axis) => AxisId::try_from(axis.id.as_str().to_owned()).unwrap(),
            Self::Time(axis) => axis.id.clone(),
            Self::Space(axis) => axis.id.clone(),
        }
    }

    pub fn size(&self) -> Option<AnyAxisSize>{
        match self {
            Self::Batch(_) => None,
            Self::Channel(axis) => Some(AnyAxisSize::Fixed(axis.size())),
            Self::Index(axis) => Some(axis.size.clone()),
            Self::Time(axis) => Some(axis.size.clone()),
            Self::Space(axis) => Some(axis.size.clone()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(try_from = "Vec::<InputAxis>")]
pub struct InputAxisGroup(Vec<InputAxis>);

impl_axis_group!(Input);
