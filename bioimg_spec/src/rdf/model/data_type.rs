use serde::{Deserialize, Serialize};

use aspartial::AsPartial;

#[derive(
    Default, Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, strum::VariantArray, strum::VariantNames, strum::Display
)]

pub enum DataType {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "float32")]
    #[default]
    Float32,
    #[serde(rename = "float64")]
    Float64,
    #[serde(rename = "uint8")]
    Uint8,
    #[serde(rename = "uint16")]
    Uint16,
    #[serde(rename = "uint32")]
    Uint32,
    #[serde(rename = "uint64")]
    Uint64,
    #[serde(rename = "int8")]
    Int8,
    #[serde(rename = "int16")]
    Int16,
    #[serde(rename = "int32")]
    Int32,
    #[serde(rename = "int64")]
    Int64,
}

impl AsPartial for DataType {
    type Partial = String;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UintDataType{
    #[serde(rename = "uint8")]
    Uint8,
    #[serde(rename = "uint16")]
    Uint16,
    #[serde(rename = "uint32")]
    Uint32,
    #[serde(rename = "uint64")]
    Uint64,
}

impl AsPartial for UintDataType {
    type Partial = String;
}


impl From<UintDataType> for DataType{
    fn from(value: UintDataType) -> Self {
        match value{
            UintDataType::Uint8 => Self::Uint8,
            UintDataType::Uint16 => Self::Uint16,
            UintDataType::Uint32 => Self::Uint32,
            UintDataType::Uint64 => Self::Uint64,
        }
    }
}
