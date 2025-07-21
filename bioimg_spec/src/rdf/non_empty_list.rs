use std::{
    borrow::Borrow, fmt::Display, num::NonZeroUsize, ops::{Deref, DerefMut}
};

use serde::{Deserialize, Serialize};

use aspartial::AsPartial;

#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
#[aspartial(newtype)]
#[serde(transparent)]
pub struct NonEmptyList<T>(Vec<T>);


impl<T: Display> Display for NonEmptyList<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (idx, item) in self.0.iter().enumerate(){
            write!(f, "{}", item)?;
            if idx != self.0.len() - 1{
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl<T> From<NonEmptyList<T>> for Vec<T>{
    fn from(value: NonEmptyList<T>) -> Self {
        value.0
    }
}

impl<T> Deref for NonEmptyList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NonEmptyList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyList<T> {
    type Error = Vec<T>;
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(value)
        } else {
            Ok(Self(value))
        }
    }
}

impl<'a, T: Clone> TryFrom<Vec<&'a T>> for NonEmptyList<T> {
    type Error = Vec<&'a T>;
    fn try_from(value: Vec<&'a T>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(value)
        } else {
            Ok(Self(value.into_iter().map(|v| v.clone()).collect()))
        }
    }
}

impl<T> Borrow<[T]> for NonEmptyList<T> {
    fn borrow(&self) -> &[T] {
        return &self.0;
    }
}

impl<T> NonEmptyList<T> {
    pub fn into_inner(self) -> Vec<T>{
        self.0
    }
    pub fn map<F, Out>(&self, f: F) -> NonEmptyList<Out>
    where
        F: FnMut(&T) -> Out,
    {
        NonEmptyList(self.iter().map(f).collect())
    }

    pub fn try_map<F, O, E>(&self, f: F) -> Result<NonEmptyList<O>, E>
    where
        F: FnMut(&T) -> Result<O, E>,
    {
        let v: Vec<O> = self.iter().map(f).collect::<Result<Vec<O>, E>>()?;
        Ok(NonEmptyList(v))
    }

    pub fn len(&self) -> NonZeroUsize{
        self.0.len().try_into().unwrap()
    }
}
