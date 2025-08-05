use serde::{Deserialize, Serialize};

use aspartial::AsPartial;

#[derive(Default, Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
#[derive(strum::EnumString, strum::VariantArray, strum::VariantNames, strum::Display)]
pub enum DataType {
    #[serde(rename = "bool")]
    #[strum(serialize = "bool")]
    Bool,
    #[serde(rename = "float32")]
    #[strum(serialize = "float32")]
    #[default]
    Float32,
    #[serde(rename = "float64")]
    #[strum(serialize = "float64")]
    Float64,
    #[serde(rename = "uint8")]
    #[strum(serialize = "uint8")]
    Uint8,
    #[serde(rename = "uint16")]
    #[strum(serialize = "uint16")]
    Uint16,
    #[serde(rename = "uint32")]
    #[strum(serialize = "uint32")]
    Uint32,
    #[serde(rename = "uint64")]
    #[strum(serialize = "uint64")]
    Uint64,
    #[serde(rename = "int8")]
    #[strum(serialize = "int8")]
    Int8,
    #[serde(rename = "int16")]
    #[strum(serialize = "int16")]
    Int16,
    #[serde(rename = "int32")]
    #[strum(serialize = "int32")]
    Int32,
    #[serde(rename = "int64")]
    #[strum(serialize = "int64")]
    Int64,
}

impl AsPartial for DataType {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(strum::EnumString, strum::Display)]
pub enum UintDataType{
    #[serde(rename = "uint8")]
    #[strum(serialize= "uint8")]
    Uint8,
    #[serde(rename = "uint16")]
    #[strum(serialize= "uint16")]
    Uint16,
    #[serde(rename = "uint32")]
    #[strum(serialize= "uint32")]
    Uint32,
    #[serde(rename = "uint64")]
    #[strum(serialize= "uint64")]
    Uint64,
}

impl AsPartial for UintDataType {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.to_string()
    }
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
