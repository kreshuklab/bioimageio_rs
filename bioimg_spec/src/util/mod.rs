use serde::{Deserialize, Serialize};
use aspartial::AsPartial;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, AsPartial)]
#[aspartial(name = PartialSingleOrMultiple)]
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
