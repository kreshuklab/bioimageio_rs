use std::fmt::Display;

use aspartial::AsPartial;
use serde::{Deserialize, Serialize};

use crate::rdf::BoundedString;
use super::HttpUrl;

#[derive(thiserror::Error, Debug)]
pub enum CiteEntryParsingError{
    #[error("Cite entry must have a DOI, a URL or both")]
    MustHaveDoiOrUrl,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, AsPartial)]
#[aspartial(name = PartialCiteEntry)]
pub struct CiteEntry {
    pub text: BoundedString<1, 1024>, //(String) free text description
    pub doi: BoundedString<1, 1024>, // FIXME: make it stricter (DOI→String) digital object identifier, see https://www.doi.org/ (alternatively specify url)
    pub url: HttpUrl,
}

#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Clone)]
#[serde(try_from="CiteEntry2Msg")]
#[serde(into="CiteEntry2Msg")]
pub struct CiteEntry2 {
    pub text: BoundedString<1, 1024>,        //(String) free text description
    doi: Option<BoundedString<1, 1024>>, // FIXME: make it stricter (DOI→String) digital object identifier, see https://www.doi.org/ (alternatively specify url)
    url: Option<HttpUrl>,
}

impl AsPartial for CiteEntry2 {
    type Partial = PartialCiteEntry2Msg;
}

impl CiteEntry2{
    pub fn doi(&self) -> Option<&BoundedString<1, 1024>>{
        self.doi.as_ref()
    }
    pub fn url(&self) -> Option<&HttpUrl>{
        self.url.as_ref()
    }
}

#[derive(serde::Serialize, serde::Deserialize, AsPartial)]
#[aspartial(name = PartialCiteEntry2Msg)]
pub struct CiteEntry2Msg{
    pub text: BoundedString<1, 1024>,        //(String) free text description
    #[serde(default)]
    pub doi: Option<BoundedString<1, 1024>>, // FIXME: make it stricter (DOI→String) digital object identifier, see https://www.doi.org/ (alternatively specify url)
    #[serde(default)]
    pub url: Option<HttpUrl>,
}

impl TryFrom<CiteEntry2Msg> for CiteEntry2{
    type Error = CiteEntryParsingError;

    fn try_from(msg: CiteEntry2Msg) -> Result<Self, Self::Error> {
        if msg.doi.is_none() && msg.url.is_none(){
            return Err(CiteEntryParsingError::MustHaveDoiOrUrl)
        }
        return Ok(CiteEntry2 { text: msg.text, doi: msg.doi, url: msg.url })
    }
}

impl From<CiteEntry2> for CiteEntry2Msg{
    fn from(value: CiteEntry2) -> Self {
        CiteEntry2Msg { text: value.text, doi: value.doi, url: value.url }
    }
}

impl Display for CiteEntry2{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl From<CiteEntry> for CiteEntry2 {
    fn from(entry1: CiteEntry) -> Self {
        CiteEntry2 {
            text: entry1.text,
            doi: Some(entry1.doi),
            url: Some(entry1.url),
        }
    }
}
