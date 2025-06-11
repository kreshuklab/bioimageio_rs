use std::{borrow::Borrow, fmt::Display};

use crate::util::AsPartial;

use super::{lowercase::Lowercase, BoundedString, EnvironmentFile, FileReference};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FileDescription<R = FileReference>
where
    R: Borrow<FileReference>
{
    pub source: R,
    pub sha256: Option<Sha256>,
}

impl < R > crate :: util :: AsPartial for FileDescription < R > where R : ::
serde :: Serialize + :: serde :: de :: DeserializeOwned + crate :: util :: AsPartial, R :
Borrow < FileReference > { type Partial = String; }

// //FIXME: generate via a serde-like macro?
// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
// pub struct PartialFileDescription {
//     pub source: Option<String>,
//     pub sha256: Option<<Sha256 as AsPartial>::Partial>,
// }

// impl AsPartial for FileDescription<FileReference>{
//     type Partial = PartialFileDescription;
// }

impl<R: Borrow<FileReference>> Display for FileDescription<R>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source.borrow())
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Sha256(Lowercase<BoundedString<64, 64>>);

impl AsPartial for Sha256 {
    type Partial = String;
}


pub type EnvironmentFileDescr = FileDescription<EnvironmentFile>;
