use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use aspartial::AsPartial;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(into = "usize")]
#[serde(try_from = "usize")]
pub struct LiteralInt<const VAL: usize>;

impl<const VAL: usize> AsPartial for LiteralInt<VAL>{
    type Partial = usize;
    fn to_partial(self) -> Self::Partial {
        VAL
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LiteralIntParsingError {
    #[error("Expected number {expected}, found '{found}'")]
    ExpectedNumberOne { expected: usize, found: usize },
}

impl<const VAL: usize> TryFrom<usize> for LiteralInt<VAL> {
    type Error = LiteralIntParsingError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value == VAL {
            Ok(Self)
        } else {
            Err(LiteralIntParsingError::ExpectedNumberOne {
                expected: VAL,
                found: value,
            })
        }
    }
}

impl<const VAL: usize> From<LiteralInt<VAL>> for usize {
    fn from(_value: LiteralInt<VAL>) -> Self {
        return VAL
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MarkerParsingError{
    #[error("Expected '{expected}', found '{found}'")]
    UnexpectedString{expected: &'static str, found: String}
}

pub trait StrMarker: Copy + Clone + Default{
    const NAME: &'static str;
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct LitStr<M: StrMarker>(#[serde(bound = "M: StrMarker")]PhantomData<M>);

impl<M: StrMarker> AsPartial for LitStr<M> {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        M::NAME.to_owned()
    }
}

impl<M: StrMarker> LitStr<M>{
    pub fn as_str(&self) -> &'static str{
        M::NAME
    }
}

impl<M: StrMarker> LitStr<M>{
    pub fn new() -> Self{ Self(PhantomData) }
}

impl<N: StrMarker> From<LitStr<N>> for String{
    fn from(_value: LitStr<N>) -> Self {
        N::NAME.to_owned()
    }
}

impl<N: StrMarker> TryFrom<String> for LitStr<N>{
    type Error = MarkerParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value == N::NAME{
            Ok(Self(PhantomData))
        }else{
            Err(MarkerParsingError::UnexpectedString { expected: N::NAME, found: value })
        }
    }
}
