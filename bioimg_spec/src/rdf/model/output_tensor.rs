use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

use crate::rdf::{model::{postprocessing::ScaleMeanVarianceDescr, preprocessing::{BinarizeDescr, ScaleLinearDescr, ScaleRangeDescr, Zmuv}, AxisId}, FileDescription};

use super::{axes::output_axes::OutputAxisGroup, postprocessing::PostprocessingDescr, TensorId, TensorTextDescription};

#[derive(thiserror::Error, Debug)]
pub enum OutputTensorParsingError{
    #[error("{tensor_id}.postprocessing[{postproc_idx}] references non-existing axis '{reference}'")]
    PostprocessingReferencesNonExistingAxis{tensor_id: TensorId, postproc_idx: usize, reference: AxisId},
    #[error("Found a self-reference from/to {tensor_id}")]
    SelfReference{tensor_id: TensorId}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputTensorDescr {
    #[serde(flatten)]
    pub metadata: OutputTensorMetadata,
    pub test_tensor: FileDescription,
    #[serde(default)]
    pub sample_tensor: Option<FileDescription>,
}


fn _default_to_output() -> TensorId{
    TensorId::try_from("output").unwrap()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "OutputTensorMetadataMsg")]
#[serde(into = "OutputTensorMetadataMsg")]
pub struct OutputTensorMetadata{
    pub id: TensorId,
    pub description: TensorTextDescription,
    axes: OutputAxisGroup,
    postprocessing: Vec<PostprocessingDescr>,
}

impl OutputTensorMetadata{
    pub fn axes(&self) -> &OutputAxisGroup{ &self.axes }
    pub fn postprocessing(&self) -> &Vec<PostprocessingDescr>{ &self.postprocessing }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutputTensorMetadataMsg{
    #[serde(default="_default_to_output")]
    pub id: TensorId,
    #[serde(default)]
    pub postprocessing: Vec<PostprocessingDescr>,
    #[serde(default)]
    pub description: TensorTextDescription,
    pub axes: OutputAxisGroup,
}

impl TryFrom<OutputTensorMetadataMsg> for OutputTensorMetadata{
    type Error = OutputTensorParsingError;
    fn try_from(message: OutputTensorMetadataMsg) -> Result<Self, Self::Error> {
        fn ensure_axis_exists(message: &OutputTensorMetadataMsg, postproc_idx: usize, postproc_axis_id: &AxisId) -> Result<(), OutputTensorParsingError>{
            message.axes.iter()
                .map(|ax| ax.id())
                .find(|ax_id| ax_id == postproc_axis_id)
                .ok_or(OutputTensorParsingError::PostprocessingReferencesNonExistingAxis{
                    tensor_id: message.id.clone(),
                    postproc_idx,
                    reference: postproc_axis_id.clone(),
                })
                .map(|_| ())
        }

        for (postproc_idx, postproc) in message.postprocessing.iter().enumerate(){
            match postproc{
                PostprocessingDescr::Binarize(BinarizeDescr::AlongAxis(descr)) => {
                    ensure_axis_exists(&message, postproc_idx, descr.axis.borrow())?;
                },
                PostprocessingDescr::ScaleLinear(ScaleLinearDescr::AlongAxis(descr)) => {
                    ensure_axis_exists(&message, postproc_idx, descr.axis.borrow())?;
                },
                PostprocessingDescr::ZeroMeanUnitVariance(Zmuv{axes: Some(axes), ..}) => {
                    for preproc_axis_id in axes.iter(){
                        ensure_axis_exists(&message, postproc_idx, preproc_axis_id)?;
                    }
                },
                PostprocessingDescr::ScaleRange(ScaleRangeDescr{axes: Some(axes), ..}) => {
                    for preproc_axis_id in axes.iter(){
                        ensure_axis_exists(&message, postproc_idx, preproc_axis_id)?;
                    }
                },
                PostprocessingDescr::ScaleMeanVarianceDescr(ScaleMeanVarianceDescr{axes: Some(axes), reference_tensor, ..}) => {
                    for preproc_axis_id in axes.iter(){
                        ensure_axis_exists(&message, postproc_idx, preproc_axis_id)?;
                    }
                    if message.id == *reference_tensor{
                        return Err(OutputTensorParsingError::SelfReference { tensor_id: message.id.clone() })
                    }
                },
                _ => (),
            }
        }
        Ok(Self{
            id: message.id,
            postprocessing: message.postprocessing,
            description: message.description,
            axes: message.axes,
        })
    }
}

impl From<OutputTensorMetadata> for OutputTensorMetadataMsg{
    fn from(value: OutputTensorMetadata) -> Self {
        Self{
            id: value.id,
            description: value.description,
            postprocessing: value.postprocessing,
            axes: value.axes,
        }
    }
}
