use aspartial::AsPartial;


#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug, strum::Display)]
pub enum RunMode{
    #[serde(rename = "imagej")]
    #[strum(serialize = "imagej")]
    ImageJ
}

impl AsPartial for RunMode {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.to_string()
    }
}
