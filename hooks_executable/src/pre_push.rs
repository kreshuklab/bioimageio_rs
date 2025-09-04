use std::str::FromStr;

use crate::refs::{GitRef, GitRefParsingError};


#[derive(Debug)]
pub struct UpdateArgs{
    pub local_ref: GitRef,
    pub local_object_name: String,
    pub remote_ref: GitRef,
    pub remote_object_name: String,
}

#[derive(Debug)]
/// Arguments passes to pre-push that signal that a ref is to be deleted
pub struct DeletionArgs{
    pub remote_ref: GitRef,
    pub remote_object_name: String,
}

pub enum PrePushEntry{
    Update(UpdateArgs),
    Deletion(DeletionArgs),
}

#[derive(thiserror::Error, Debug)]
pub enum PrePushArgsParsingError{
    #[error("Too few arguments: Expecting 4 arguments for pre-push")]
    TooFewArguments,
    #[error("Missing local ref")]
    MissingLocalRef,
    #[error("Missing local object name")]
    MissingLocalObjectName,
    #[error("Missing remote ref")]
    MissingRemoteRef,
    #[error("Missing remote object name")]
    MissingRemoteObjectName,
    #[error("Unexpected extra arguments")]
    UnexpectedExtraArguments,
    #[error("Could not parse reference: {0}")]
    GitRefParsingError(#[from] GitRefParsingError),
}

impl FromStr for UpdateArgs {
    type Err = PrePushArgsParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(" ");

        let out = Self{
            local_ref: parts.next().ok_or(Self::Err::MissingLocalRef)?.parse()?,
            local_object_name: parts.next().ok_or(Self::Err::MissingLocalObjectName)?.to_owned(),
            remote_ref: parts.next().ok_or(Self::Err::MissingRemoteRef)?.parse()?,
            remote_object_name: parts.next().ok_or(Self::Err::MissingRemoteObjectName)?.to_owned(),
        };
        if parts.next().is_some() {
            return Err(Self::Err::UnexpectedExtraArguments)
        }
        Ok(out)
    }
}
