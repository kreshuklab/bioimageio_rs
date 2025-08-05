use serde::{Deserialize, Serialize};
use aspartial::AsPartial;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum SingleOrMultiple<T> {
    Single(T),
    Multiple(Vec<T>),
}

impl<T: AsPartial> AsPartial for SingleOrMultiple<T> {
    type Partial = PartialSingleOrMultiple<T>;
    fn to_partial(self) -> Self::Partial {
        match self {
            Self::Single(v) => PartialSingleOrMultiple::Single(v.to_partial()),
            Self::Multiple(vs) => PartialSingleOrMultiple::Multiple(
                vs.into_iter().map(|v| v.to_partial()).collect()
            ),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum PartialSingleOrMultiple<T: AsPartial> {
    Single(T::Partial),
    Multiple(Vec<T::Partial>),
}

impl<T: AsPartial> AsPartial for PartialSingleOrMultiple<T> {
    type Partial = Self;
    fn to_partial(self) -> Self::Partial {
        self
    }
}

impl<T> SingleOrMultiple<T> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Single(t) => std::slice::from_ref(t),
            Self::Multiple(ts) => ts,
        }
    }
}
