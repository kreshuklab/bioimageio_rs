use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{rdf::BoundedString, util::AsPartial};

use super::orcid::Orcid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Author {
    pub name: BoundedString<1, 1024>,        // (Name→String) Full name. FIXME: disallow / and \.
    pub affiliation: BoundedString<1, 1024>, // (String) Affiliation.
    pub email: BoundedString<1, 1024>,       // FIXME: make a parser here (Email) E-Mail
    pub github_user: BoundedString<1, 1024>, // (String) GitHub user name.
    pub orcid: Orcid,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, AsPartial)]
pub struct Author2 {
    pub name: BoundedString<1, 1024>,                // (Name→String) Full name.
    pub affiliation: Option<BoundedString<1, 1024>>, // (String) Affiliation.
    pub email: Option<BoundedString<1, 1024>>,       // FIXME: make a parser here (Email) E-Mail
    pub github_user: Option<BoundedString<1, 1024>>, // (String) GitHub user name.
    pub orcid: Option<Orcid>,
}

impl Display for Author2{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(email) = &self.email{
            write!(f, " 📧{email}")?;
        }
        if let Some(github_user) = &self.github_user{
            write!(f, " : {github_user}")?;
        }
        Ok(())
    }
}

impl From<Author> for Author2 {
    fn from(value: Author) -> Self {
        Author2 {
            name: value.name,
            affiliation: Some(value.affiliation),
            email: Some(value.email),
            github_user: Some(value.github_user),
            orcid: Some(value.orcid),
        }
    }
}
