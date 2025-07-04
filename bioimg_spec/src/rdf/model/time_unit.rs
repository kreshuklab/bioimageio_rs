use serde::{Serialize, Deserialize};

use crate::util::AsPartial;

#[derive(
    Default, Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone, strum::VariantArray, strum::VariantNames, strum::Display
)]
pub enum TimeUnit{
    #[serde(rename = "attosecond")]
    #[strum(to_string = "attosecond")]
    Attosecond,
    #[serde(rename = "centisecond")]
    #[strum(to_string = "centisecond")]
    Centisecond,
    #[serde(rename = "day")]
    #[strum(to_string = "day")]
    Day,
    #[serde(rename = "decisecond")]
    #[strum(to_string = "decisecond")]
    Decisecond,
    #[serde(rename = "exasecond")]
    #[strum(to_string = "exasecond")]
    Exasecond,
    #[serde(rename = "femtosecond")]
    #[strum(to_string = "femtosecond")]
    Femtosecond,
    #[serde(rename = "gigasecond")]
    #[strum(to_string = "gigasecond")]
    Gigasecond,
    #[serde(rename = "hectosecond")]
    #[strum(to_string = "hectosecond")]
    Hectosecond,
    #[serde(rename = "hour")]
    #[strum(to_string = "hour")]
    Hour,
    #[serde(rename = "kilosecond")]
    #[strum(to_string = "kilosecond")]
    Kilosecond,
    #[serde(rename = "megasecond")]
    #[strum(to_string = "megasecond")]
    Megasecond,
    #[serde(rename = "microsecond")]
    #[strum(to_string = "microsecond")]
    Microsecond,
    #[serde(rename = "millisecond")]
    #[strum(to_string = "millisecond")]
    Millisecond,
    #[serde(rename = "minute")]
    #[strum(to_string = "minute")]
    Minute,
    #[serde(rename = "nanosecond")]
    #[strum(to_string = "nanosecond")]
    Nanosecond,
    #[serde(rename = "petasecond")]
    #[strum(to_string = "petasecond")]
    Petasecond,
    #[serde(rename = "picosecond")]
    #[strum(to_string = "picosecond")]
    Picosecond,
    #[default] //FIXME: should this have a default to begin with?
    #[serde(rename = "second")]
    #[strum(to_string = "second")]
    Second,
    #[serde(rename = "terasecond")]
    #[strum(to_string = "terasecond")]
    Terasecond,
    #[serde(rename = "yoctosecond")]
    #[strum(to_string = "yoctosecond")]
    Yoctosecond,
    #[serde(rename = "yottasecond")]
    #[strum(to_string = "yottasecond")]
    Yottasecond,
    #[serde(rename = "zeptosecond")]
    #[strum(to_string = "zeptosecond")]
    Zeptosecond,
    #[serde(rename = "zettasecond")]
    #[strum(to_string = "zettasecond")]
    Zettasecond,
}

impl AsPartial for TimeUnit {
    type Partial = String;
}
