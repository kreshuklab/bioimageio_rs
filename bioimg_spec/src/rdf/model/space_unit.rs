use serde::{Serialize, Deserialize};

use aspartial::AsPartial;

#[derive(Default, Serialize, Deserialize, Eq, PartialEq, Debug, Copy, Clone)]
#[derive(strum::VariantArray, strum::VariantNames, strum::EnumString, strum::Display)]
pub enum SpaceUnit{
    #[serde(rename = "attometer")]
    #[strum(to_string = "attometer")]
    Attometer,
    #[serde(rename = "angstrom")]
    #[strum(to_string = "angstrom")]
    Angstrom,
    #[serde(rename = "centimeter")]
    #[strum(to_string = "centimeter")]
    Centimeter,
    #[serde(rename = "decimeter")]
    #[strum(to_string = "decimeter")]
    Decimeter,
    #[serde(rename = "exameter")]
    #[strum(to_string = "exameter")]
    Exameter,
    #[serde(rename = "femtometer")]
    #[strum(to_string = "femtometer")]
    Femtometer,
    #[serde(rename = "foot")]
    #[strum(to_string = "foot")]
    Foot,
    #[serde(rename = "gigameter")]
    #[strum(to_string = "gigameter")]
    Gigameter,
    #[serde(rename = "hectometer")]
    #[strum(to_string = "hectometer")]
    Hectometer,
    #[serde(rename = "inch")]
    #[strum(to_string = "inch")]
    Inch,
    #[serde(rename = "kilometer")]
    #[strum(to_string = "kilometer")]
    Kilometer,
    #[serde(rename = "megameter")]
    #[strum(to_string = "megameter")]
    Megameter,
    #[serde(rename = "meter")]
    #[strum(to_string = "meter")]
    Meter,
    #[default]
    #[serde(rename = "micrometer")]
    #[strum(to_string = "micrometer")]
    Micrometer,
    #[serde(rename = "mile")]
    #[strum(to_string = "mile")]
    Mile,
    #[serde(rename = "millimeter")]
    #[strum(to_string = "millimeter")]
    Millimeter,
    #[serde(rename = "nanometer")]
    #[strum(to_string = "nanometer")]
    Nanometer,
    #[serde(rename = "parsec")]
    #[strum(to_string = "parsec")]
    Parsec,
    #[serde(rename = "petameter")]
    #[strum(to_string = "petameter")]
    Petameter,
    #[serde(rename = "picometer")]
    #[strum(to_string = "picometer")]
    Picometer,
    #[serde(rename = "terameter")]
    #[strum(to_string = "terameter")]
    Terameter,
    #[serde(rename = "yard")]
    #[strum(to_string = "yard")]
    Yard,
    #[serde(rename = "yoctometer")]
    #[strum(to_string = "yoctometer")]
    Yoctometer,
    #[serde(rename = "yottameter")]
    #[strum(to_string = "yottameter")]
    Yottameter,
    #[serde(rename = "zeptometer")]
    #[strum(to_string = "zeptometer")]
    Zeptometer,
    #[serde(rename = "zettameter")]
    #[strum(to_string = "zettameter")]
    Zettameter,
}

impl AsPartial for SpaceUnit {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.to_string()
    }
}
