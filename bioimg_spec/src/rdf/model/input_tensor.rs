use std::borrow::Borrow;

use aspartial::AsPartial;
use serde::{Deserialize, Serialize};

use crate::rdf::FileDescription;
use super::{axes::input_axes::InputAxisGroup, preprocessing::{BinarizeDescr, PreprocessingDescr, ScaleLinearDescr, ScaleRangeDescr, Zmuv}, AxisId, TensorId, TensorTextDescription};

#[derive(thiserror::Error, Debug)]
pub enum InputTensorParsingError{
    #[error("{tensor_id}.preprocessing[{preproc_idx}] references non-existing axis '{reference}'")]
    PreprocessingReferencesNonExistingAxis{tensor_id: TensorId, preproc_idx: usize, reference: AxisId},
}


#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialInputTensorDescr)]
pub struct InputTensorDescr {
    #[serde(flatten)]
    pub meta: InputTensorMetadata,
    pub test_tensor: FileDescription,
    pub sample_tensor: Option<FileDescription>,
}

// pub struct PartialInputTensorDescr {
//     pub meta: InputTensorMetadata,
//     pub test_tensor: FileDescription,
//     pub sample_tensor: Option<FileDescription>,
// }


#[derive(Serialize, Deserialize, Debug, Clone, AsPartial)]
#[aspartial(name = PartialInputTensorMetadata )]
#[serde(into = "InputTensorMetadataMsg")]
#[serde(try_from = "InputTensorMetadataMsg")]
pub struct InputTensorMetadata {
    /// Input tensor id. No duplicates are allowed across all inputs and outputs.
    pub id: TensorId,

    /// indicates that this tensor is optional when doing inference
    #[serde(default)]
    pub optional: bool,

    pub description: TensorTextDescription,

    /// Description of how this input should be preprocessed.
    ///
    /// notes:
    /// - If preprocessing does not start with an 'ensure_dtype' entry, it is added
    ///   to ensure an input tensor's data type matches the input tensor's data description.
    /// - If preprocessing does not end with an 'ensure_dtype' or 'binarize' entry, an
    ///   'ensure_dtype' step is added to ensure preprocessing steps are not unintentionally
    ///   changing the data type.
    preprocessing: Vec<PreprocessingDescr>,
    axes: InputAxisGroup,
}

impl InputTensorMetadata{
    pub fn axes(&self) -> &InputAxisGroup{ &self.axes }
    pub fn preprocessing(&self) -> &Vec<PreprocessingDescr>{ &self.preprocessing }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputTensorMetadataMsg {
    pub id: TensorId,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub preprocessing: Vec<PreprocessingDescr>,
    #[serde(default)]
    pub description: TensorTextDescription,
    pub axes: InputAxisGroup,
}

impl TryFrom<InputTensorMetadataMsg> for InputTensorMetadata{
    type Error = InputTensorParsingError;
    fn try_from(message: InputTensorMetadataMsg) -> Result<Self, Self::Error> {

        fn ensure_axis_exists(message: &InputTensorMetadataMsg, preproc_idx: usize, preproc_axis_id: &AxisId) -> Result<(), InputTensorParsingError>{
            message.axes.iter()
                .map(|ax| ax.id())
                .find(|ax_id| {
                    ax_id == preproc_axis_id
                })
                .ok_or(InputTensorParsingError::PreprocessingReferencesNonExistingAxis{
                    tensor_id: message.id.clone(),
                    preproc_idx,
                    reference: preproc_axis_id.clone()
                })
                .map(|_| ())
        }

        for (preproc_idx, preproc) in message.preprocessing.iter().enumerate(){
            match preproc{
                PreprocessingDescr::Binarize(BinarizeDescr::AlongAxis(descr)) => {
                    ensure_axis_exists(&message, preproc_idx, descr.axis.borrow())?;
                },
                PreprocessingDescr::ScaleLinear(ScaleLinearDescr::AlongAxis(descr)) => {
                    ensure_axis_exists(&message, preproc_idx, descr.axis.borrow())?;
                },
                PreprocessingDescr::ZeroMeanUnitVariance(Zmuv{axes: Some(axes), ..}) => {
                    for preproc_axis_id in axes.iter(){
                        ensure_axis_exists(&message, preproc_idx, preproc_axis_id)?;
                    }
                },
                PreprocessingDescr::ScaleRange(ScaleRangeDescr{axes: Some(axes), ..}) => {
                    for preproc_axis_id in axes.iter(){
                        ensure_axis_exists(&message, preproc_idx, preproc_axis_id)?;
                    }
                },
                _ => (),
            }
        }

        Ok(Self{
            id: message.id,
            optional: message.optional,
            preprocessing: message.preprocessing,
            description: message.description,
            axes: message.axes,
        })
    }
}

impl From<InputTensorMetadata> for InputTensorMetadataMsg{
    fn from(value: InputTensorMetadata) -> Self {
        Self{
            id: value.id,
            optional: value.optional,
            preprocessing: value.preprocessing,
            description: value.description,
            axes: value.axes,
        }
    }
}

