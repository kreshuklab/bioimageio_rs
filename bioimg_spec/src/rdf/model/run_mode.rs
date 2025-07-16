use aspartial::AsPartial;


#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum RunMode{
    #[serde(rename = "imagej")]
    ImageJ
}

impl AsPartial for RunMode {
    type Partial = String;
}
