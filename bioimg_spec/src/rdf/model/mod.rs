use ::aspartial::AsPartial;

use super::basic_chars_string::BasicCharsString;
use super::BoundedString;

pub mod axes;
pub mod axis_size;
pub mod data_range;
pub mod data_type;
pub mod input_tensor;
pub mod output_tensor;
pub mod preprocessing;
pub mod postprocessing;
pub mod space_unit;
pub mod tensor_data_descr;
pub mod tensor_id;
pub mod time_unit;
pub mod weights;
pub mod run_mode;
pub mod dataset_descr;
pub mod unsupported;
pub mod model_rdf_0_5;
// pub mod model_rdf_0_4;

pub use axes::{
    AxisType, AxisId, AxisScale,
    BatchAxis, ChannelAxis, IndexAxis,
    Batch, Index, Channel, Space, Time,
    Halo,
};
pub use axes::input_axes::{InputAxis, InputAxisGroup, SpaceInputAxis, TimeInputAxis};
pub use axes::output_axes::{OutputAxis, OutputAxisGroup, SpaceOutputAxis, TimeOutputAxis};
pub use axis_size::{AnyAxisSize, AxisSizeReference, FixedAxisSize, ParameterizedAxisSize, QualifiedAxisId, ResolvedAxisSize};
pub use input_tensor::InputTensorDescr;
pub use output_tensor::OutputTensorDescr;
pub use space_unit::SpaceUnit;
pub use tensor_id::TensorId;
pub use time_unit::TimeUnit;
pub use weights::{
    WeightsDescr,
    MaybeSomeWeightsDescr,
    KerasHdf5WeightsDescr,
    WeightsDescrBase,
    OnnxWeightsDescr,
    OnnxOpsetVersion,
    PytorchStateDictWeightsDescr,
    PytorchArchitectureDescr,
    TensorflowJsWeightsDescr,
    TensorflowSavedModelBundleWeightsDescr,
    TorchscriptWeightsDescr,
};
pub use preprocessing::PreprocessingDescr;
pub use data_type::DataType;
pub use model_rdf_0_5::ModelRdfV0_5;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct RdfTypeModel;

impl AsPartial for RdfTypeModel {
    type Partial = String;
}

impl From<RdfTypeModel> for String{
    fn from(_: RdfTypeModel) -> Self {
        return "model".into()
    }
}

impl TryFrom<String> for RdfTypeModel{
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value == "model"{
            Ok(Self)
        }else{
            Err(value)
        }
    }
}

fn _now() -> iso8601_timestamp::Timestamp{
    iso8601_timestamp::Timestamp::now_utc()
}

pub type TensorTextDescription = BoundedString<0, 128>;
pub type ModelRdfName = BasicCharsString<BoundedString<5, 128>>;
