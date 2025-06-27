use std::sync::Arc;

pub use bioimg_codegen::AsPartial;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

impl<T> SingleOrMultiple<T> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Single(t) => std::slice::from_ref(t),
            Self::Multiple(ts) => ts,
        }
    }
}

pub trait AsPartial{
    type Partial;
}

pub trait AsSerializablePartial: AsPartial<Partial: serde::Serialize + serde::de::DeserializeOwned>
{}

impl<T> AsSerializablePartial for T
where T: AsPartial<Partial: serde::Serialize + serde::de::DeserializeOwned>
{}

impl AsPartial for String{
    type Partial = String;
}

impl AsPartial for Arc<str>{
    type Partial = String;
}

impl AsPartial for f32 {
    type Partial = f32;
}

//FIXME: T::Partial and not Option<T::Partial>??
impl<T: AsPartial> AsPartial for Option<T>{
    type Partial = T::Partial;
}



