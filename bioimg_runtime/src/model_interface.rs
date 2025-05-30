use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Display;
use std::io::{Seek, Write};
use std::sync::Arc;

use bioimg_spec::rdf;
use bioimg_spec::rdf::model::postprocessing::PostprocessingDescr;
use bioimg_spec::rdf::model::preprocessing::ScaleRangeDescr;
use ndarray_npy::ReadNpyError;

use crate::axis_size_resolver::{ResolvedAxisSizeExt, SlotResolver};
use crate::file_source::FileSourceError;
use crate::npy_array::NpyArray;
use crate::zip_archive_ext::SharedZipArchive;
use crate::zip_writer_ext::ModelZipWriter;
use crate::zoo_model::ModelPackingError;
use crate::FileSource;
use bioimg_spec::rdf::model::axis_size::QualifiedAxisId;
use bioimg_spec::rdf::model::{AnyAxisSize, InputAxis, OutputAxis, PreprocessingDescr};
use bioimg_spec::rdf::model::{self as modelrdf, TensorId};

use super::axis_size_resolver::AxisSizeResolutionError;

#[derive(thiserror::Error, Debug)]
pub enum ModelInterfaceLoadingError{
    #[error(transparent)]
    FileSourceOpenError(#[from] FileSourceError),
    #[error(transparent)]
    ReadNpyError(#[from] ReadNpyError),
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct InputSlot <DATA: Borrow<NpyArray>> {
    pub tensor_meta: modelrdf::input_tensor::InputTensorMetadata,
    pub test_tensor: DATA,
}

impl<DATA: Borrow<NpyArray>> Display for InputSlot<DATA>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}([", self.tensor_meta.id)?;
        for (idx, axis) in self.tensor_meta.axes().iter().enumerate(){
            write!(f, "{}", axis.id())?;
            if idx != self.tensor_meta.axes().len() - 1{
                write!(f, ", ")?;
            }
        }
        write!(f, "])")?;
        if !self.tensor_meta.preprocessing().is_empty(){
            write!(f, " (preprocessed)")?;
        }
        Ok(())
    }
}

impl InputSlot<Arc<NpyArray>> {
    pub fn dump(
        &self,
        zip_file: &mut ModelZipWriter<impl Write + Seek>,
    ) -> Result<modelrdf::InputTensorDescr, ModelPackingError> {
        let test_tensor_zip_path = rdf::FsPath::unique_suffixed(&format!("_{}_test_tensor.npy", self.tensor_meta.id));
        zip_file.write_file(&test_tensor_zip_path, |writer| self.test_tensor.write_npy(writer))?;
        Ok(modelrdf::input_tensor::InputTensorDescr{
            meta: self.tensor_meta.clone(),
            test_tensor: rdf::FileDescription{
                source: test_tensor_zip_path.into(),
                sha256: None,
            },
            sample_tensor: None, //FIXME
        })
    }

    pub fn try_from_rdf(
        rdf: modelrdf::InputTensorDescr, archive: SharedZipArchive
    ) -> Result<Self, ModelInterfaceLoadingError>{
        let mut test_tensor_raw_data = vec![];
        FileSource::from_rdf_file_descr(archive, &rdf.test_tensor)?.read_to_end(&mut test_tensor_raw_data)?;
        //FIXME: there's another copy inside try_load, i think
        let test_tensor = NpyArray::try_load(&mut test_tensor_raw_data.as_slice())?;

        Ok(Self{
            tensor_meta: rdf.meta,
            test_tensor: Arc::new(test_tensor),
        })
    }
}

pub trait VecInputSlotExt{
    fn qual_id_axes(&self) -> impl Iterator<Item=(QualifiedAxisId, &InputAxis)>;
    fn qual_id_sizes(&self) -> impl Iterator<Item=(QualifiedAxisId, AnyAxisSize)>{
        self.qual_id_axes().filter_map(|(qual_id, axis)| axis.size().map(|size| (qual_id, size)))
    }
}
impl<DATA: Borrow<NpyArray>> VecInputSlotExt for [InputSlot<DATA>]{
    fn qual_id_axes(&self) -> impl Iterator<Item=(QualifiedAxisId, &InputAxis)>{
        self.iter()
            .map(|slot|{
                slot.tensor_meta.axes().iter().map(|axis|{
                    let qual_id = QualifiedAxisId{tensor_id: slot.tensor_meta.id.clone(), axis_id: axis.id()};
                    (qual_id, axis)
                })
            })
            .flatten()
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct OutputSlot<DATA: Borrow<NpyArray>> {
    pub tensor_meta: modelrdf::output_tensor::OutputTensorMetadata,
    pub test_tensor: DATA,
}

impl<DATA: Borrow<NpyArray>> Display for OutputSlot<DATA>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}([", self.tensor_meta.id)?;
        for (idx, axis) in self.tensor_meta.axes().iter().enumerate(){
            write!(f, "{}", axis.id())?;
            if idx != self.tensor_meta.axes().len() - 1{
                write!(f, ", ")?;
            }
        }
        write!(f, "])")?;
        if !self.tensor_meta.postprocessing().is_empty(){
            write!(f, " (postprocessed)")?;
        }
        Ok(())
    }
}

impl OutputSlot<Arc<NpyArray>> {
    pub fn dump(
        &self,
        zip_file: &mut ModelZipWriter<impl Write + Seek>,
    ) -> Result<modelrdf::OutputTensorDescr, ModelPackingError> {
        let test_tensor_zip_path = rdf::FsPath::unique_suffixed(&format!("_{}_test_tensor.npy", self.tensor_meta.id));
        zip_file.write_file(&test_tensor_zip_path, |writer| self.test_tensor.write_npy(writer))?;
        Ok(modelrdf::OutputTensorDescr{
            metadata: self.tensor_meta.clone(),
            test_tensor: rdf::FileDescription{
                source: test_tensor_zip_path.into(),
                sha256: None,
            },
            sample_tensor: None, //FIXME
        })
    }

    pub fn try_from_rdf(
        rdf: modelrdf::OutputTensorDescr, archive: SharedZipArchive,
    ) -> Result<Self, ModelInterfaceLoadingError>{
        let mut test_tensor_raw_data = vec![];
        FileSource::from_rdf_file_descr(archive, &rdf.test_tensor)?.read_to_end(&mut test_tensor_raw_data)?;
        //FIXME: there's another copy inside try_load, i think
        let test_tensor = NpyArray::try_load(&mut test_tensor_raw_data.as_slice())?;
        Ok(Self{
            tensor_meta: rdf.metadata,
            test_tensor: Arc::new(test_tensor),
        })
    }
}

pub trait VecOutputSlotExt{
    fn qual_id_axes(&self) -> impl Iterator<Item=(QualifiedAxisId, &OutputAxis)>;
    fn qual_id_sizes(&self) -> impl Iterator<Item=(QualifiedAxisId, AnyAxisSize)>{
        self.qual_id_axes().filter_map(|(qual_id, axis)| axis.size().map(|size| (qual_id, size)))
    }
}
impl<DATA: Borrow<NpyArray>> VecOutputSlotExt for [OutputSlot<DATA>]{
    fn qual_id_axes(&self) -> impl Iterator<Item=(QualifiedAxisId, &OutputAxis)>{
        self.iter()
            .map(|slot|{
                slot.tensor_meta.axes().iter().map(|axis|{
                    let qual_id = QualifiedAxisId{tensor_id: slot.tensor_meta.id.clone(), axis_id: axis.id()};
                    (qual_id, axis)
                })
            })
            .flatten()
    }
}


#[derive(thiserror::Error, Debug)]
pub enum TensorValidationError {
    #[error("{0}")]
    ReadNpyError(#[from] ndarray_npy::ReadNpyError),
    #[error("Urls file references are unsupported for now")]
    UrlUnsupported,
    #[error("Test tensor with shape {test_tensor_shape:?} does not map number of reported axes ({num_described_axes})")]
    MismatchedNumDimensions {
        test_tensor_shape: Vec<usize>,
        num_described_axes: usize,
    },
    #[error(
        "Axis '{qualified_axis_id}' is incompatible with test tensor dim #{test_tensor_dim_index} with extent {test_tensor_dim_size}"
    )]
    IncompatibleAxis {
        qualified_axis_id: QualifiedAxisId,
        test_tensor_dim_size: usize,
        test_tensor_dim_index: usize,
    },
    #[error("{0}")]
    AxisSizeResolutionError(#[from] AxisSizeResolutionError),
    #[error("Duplicate tensor id: {0}")]
    DuplicateTensorId(TensorId),
    #[error("Empty model interface inputs")]
    EmptyInputs,
    #[error("Empty model interface outputs")]
    EmptyOutputs,
    #[error("No tensor with ID {reference}")]
    InvalidTensorReference{reference: TensorId}
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ModelInterface<DATA: Borrow<NpyArray>> {
    inputs: rdf::NonEmptyList<InputSlot<DATA>>,
    outputs: rdf::NonEmptyList<OutputSlot<DATA>>,
}

impl ModelInterface<Arc<NpyArray>> {
    pub fn dump(
        &self,
        zip_writer: &mut ModelZipWriter<impl Write + Seek>,
    ) -> Result<
        (
            rdf::NonEmptyList<modelrdf::InputTensorDescr>,
            rdf::NonEmptyList<modelrdf::OutputTensorDescr>,
        ),
        ModelPackingError,
    > {
        let inputs = self.inputs.try_map(|inp| inp.dump(zip_writer))?;
        let outputs = self.outputs.try_map(|out| out.dump(zip_writer))?;
        Ok((inputs, outputs))
    }
}

impl<DATA: Borrow<NpyArray>> ModelInterface<DATA> {
    pub fn inputs(&self) -> &rdf::NonEmptyList<InputSlot<DATA>>{
        &self.inputs
    }
    pub fn outputs(&self) -> &rdf::NonEmptyList<OutputSlot<DATA>>{
        &self.outputs
    }
    pub fn try_build(inputs: Vec<InputSlot<DATA>>, outputs: Vec<OutputSlot<DATA>>) -> Result<Self, TensorValidationError> {
        let inputs = rdf::NonEmptyList::try_from(inputs).map_err(|_| TensorValidationError::EmptyInputs)?;
        let outputs = rdf::NonEmptyList::try_from(outputs).map_err(|_| TensorValidationError::EmptyOutputs)?;

        let capacity: usize = usize::from(inputs.len()) + usize::from(outputs.len());
        let mut seen_tensor_ids = HashSet::<&TensorId>::with_capacity(capacity);
        {
            inputs.iter().map(|slot| &slot.tensor_meta.id)
                .chain(outputs.iter().map(|slot| &slot.tensor_meta.id))
                .map(|tensor_id|{
                    if !seen_tensor_ids.insert(tensor_id){
                        Err(TensorValidationError::DuplicateTensorId(tensor_id.clone()))
                    }else{
                        Ok(())
                    }
                })
                .collect::<Result<(), TensorValidationError>>()?;
        }

        let axis_sizes: Vec<(QualifiedAxisId, AnyAxisSize)> = inputs.qual_id_sizes()
            .chain(outputs.qual_id_sizes())
            .collect();

        let size_map = SlotResolver::new(axis_sizes)?.solve()?;

        macro_rules! validate_resolution {( $slots:ident ) => {
            for slot in $slots.iter(){
                let test_tensor_shape = slot.test_tensor.borrow().shape();
                let mut test_tensor_dims = test_tensor_shape.iter().enumerate();
                for axis in slot.tensor_meta.axes().iter(){
                    let Some((test_tensor_dim_index, test_tensor_dim_size)) = test_tensor_dims.next() else{
                        return Err(TensorValidationError::MismatchedNumDimensions {
                            test_tensor_shape: test_tensor_shape.into(),
                            num_described_axes: slot.tensor_meta.axes().len(),
                        });
                    };
                    if axis.size().is_none(){ // batch i guess?
                        continue;
                    };
                    let qual_id = QualifiedAxisId{tensor_id: slot.tensor_meta.id.clone(), axis_id: axis.id()};
                    let resolved = size_map.get(&qual_id).unwrap();
                    if !resolved.is_compatible_with_extent(*test_tensor_dim_size){
                        return Err(TensorValidationError::IncompatibleAxis {
                            qualified_axis_id: qual_id,
                            test_tensor_dim_index,
                            test_tensor_dim_size: *test_tensor_dim_size,
                        });
                    }
                }
            }
        };}
        validate_resolution!(inputs);
        validate_resolution!(outputs);

        for input in inputs.iter(){
            for proc in input.tensor_meta.preprocessing() {
                let tensor_ref = match proc{
                    PreprocessingDescr::ScaleRange(ScaleRangeDescr{reference_tensor: Some(tensor_ref), ..}) => tensor_ref,
                    _ => continue,
                };
                if inputs.iter().find(|inp| inp.tensor_meta.id == *tensor_ref).is_none(){
                    return Err(TensorValidationError::InvalidTensorReference{reference: tensor_ref.clone()})
                }
            }
        }
        for output in outputs.iter(){
            for proc in output.tensor_meta.postprocessing() {
                let tensor_ref = match proc{
                    PostprocessingDescr::ScaleMeanVarianceDescr(descr) => &descr.reference_tensor,
                    PostprocessingDescr::ScaleRange(ScaleRangeDescr{reference_tensor: Some(tensor_ref), ..}) => tensor_ref,
                    _ => continue,
                };
                if !seen_tensor_ids.contains(tensor_ref){
                    return Err(TensorValidationError::InvalidTensorReference{reference: tensor_ref.clone()})
                }
            }
        }

        Ok(Self{inputs, outputs})
    }
}
