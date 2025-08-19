use std::fmt::Display;

use ::aspartial::AsPartial;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, AsPartial)]
#[aspartial(name = PartialSigmoid)]
pub struct Sigmoid;

impl Display for Sigmoid{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sigmoid")
    }
}
