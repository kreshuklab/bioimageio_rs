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

/// Types that represent some structure in a serialized payload can implement
/// AsPartial to specify what that structure would look like when incomplete.
/// Usually, for a struct of the form
/// ```rust
/// struct MyStruct{
///   field1: Something,
///   field2: SomethingElse,
/// }
/// ```
/// a partial version of it would be of the form
/// ```rust
/// struct PartialMyStruct{
///   field1: Option<<Something as AsPartial>::Partial>,
///   field2: Option<<Something as AsPartial>::Partial>,
/// }
/// ```
///
///which expresses some data that has the same structure as MyStruct but
/// maybe have some (or all) of its (arbitrarily nested) fields missing.
/// 
/// Note that [AsPartial::Partial] also implements [AsPartial], so that
/// any arbitrarily nested field can also be missing
pub trait AsPartial{
    type Partial: AsPartial;
}

/// Partial types are mostly useful in the context of (de)serialization, to be able
/// to handle incomplete data in self-describing formats (e.g. JSON, YAML).
/// For convenience, the AsSerializablePartial is blanket-implemented for all types
/// that implement `AsPartial` and whose partial version is also serializable
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

impl AsPartial for (f32, f32) {
    type Partial = (f32, f32);
}

//FIXME: T::Partial and not Option<T::Partial>??
impl<T: AsPartial> AsPartial for Option<T>{
    type Partial = T::Partial;
}

impl<T: AsPartial> AsPartial for Vec<T> {
    type Partial = Vec<T::Partial>;
}

impl AsPartial for serde_json::Map<String, serde_json::Value>{
    type Partial = Self;
}

impl AsPartial for bool {
    type Partial = bool;
}
