use std::fmt::Display;

use serde::{Deserialize, Serialize};

use aspartial::AsPartial;

use super::{bounded_string::BoundedString, orcid::Orcid, slashless_string::SlashlessString};

pub type MaintainerName = SlashlessString<BoundedString<1, 1024>>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, AsPartial)]
#[aspartial(name = PartialMaintainer)]
pub struct Maintainer {
    pub affiliation: Option<BoundedString<1, 1024>>,
    pub email: Option<BoundedString<1, 1024>>, //FIXME
    pub orcid: Option<Orcid>,
    pub name: Option<MaintainerName>,
    pub github_user: BoundedString<1, 1024>, //FIXME validate this somehow
}

impl Display for Maintainer{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name{
            write!(f, "{name} ")?;
        };
        write!(f, " : {}", self.github_user)?;
        if let Some(email) = &self.email{
            write!(f, " 📧{email}")?;
        }
        Ok(())
    }
}
