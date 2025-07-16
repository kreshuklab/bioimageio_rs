use std::ops::Deref;

use aspartial::AsPartial;

use crate::rdf::{author::Author2, file_description::{FileDescription, Sha256}, file_reference::EnvironmentFile, FileReference, Identifier, Version};

#[derive(thiserror::Error, Debug, Clone)]
pub enum ModelWeightsParsingError{
    #[error("Bad or unsupported Onnx opset version: {0}. Must be >= 7")]
    BadOnnxOpsetVersion(u32),
    #[error("No model weights found")]
    NoWeightsFound,
    #[error("Dependencies must be a .yml or .yaml file")]
    DependenciesNotYaml{path: String}
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, AsPartial)]
#[aspartial(name = PartialMaybeSomeWeightsDescr)]
pub struct MaybeSomeWeightsDescr{
    #[serde(default)]
    pub keras_hdf5: Option<KerasHdf5WeightsDescr>,
    #[serde(default)]
    pub onnx: Option<OnnxWeightsDescr>,
    #[serde(default)]
    pub pytorch_state_dict: Option<PytorchStateDictWeightsDescr>,
    #[serde(default)]
    pub tensorflow_js: Option<TensorflowJsWeightsDescr>,
    #[serde(default)]
    pub tensorflow_saved_model_bundle: Option<TensorflowSavedModelBundleWeightsDescr>,
    #[serde(default)]
    pub torchscript: Option<TorchscriptWeightsDescr>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(try_from = "MaybeSomeWeightsDescr")]
pub struct WeightsDescr(MaybeSomeWeightsDescr);

impl AsPartial for WeightsDescr {
    type Partial = <MaybeSomeWeightsDescr as AsPartial>::Partial;
}

impl WeightsDescr{
    pub fn into_inner(self) -> MaybeSomeWeightsDescr{
        self.0
    }
}

impl Deref for WeightsDescr{
    type Target = MaybeSomeWeightsDescr;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<MaybeSomeWeightsDescr> for WeightsDescr{
    type Error = ModelWeightsParsingError;
    fn try_from(value: MaybeSomeWeightsDescr) -> Result<Self, Self::Error> {
        if value.keras_hdf5.is_none()
        && value.onnx.is_none()
        && value.pytorch_state_dict.is_none()
        && value.tensorflow_js.is_none()
        && value.tensorflow_saved_model_bundle.is_none()
        && value.torchscript.is_none(){
            return Err(ModelWeightsParsingError::NoWeightsFound)
        }
        Ok(Self(value))
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum WeightsFormat{
    #[serde(rename = "keras_hdf5")]
    KerasHdf5,
    #[serde(rename="onnx")]
    Onnx,
    #[serde(rename="pytorch_state_dict")]
    PytorchStateDict,
    #[serde(rename="tensorflow_js")]
    TensorflowJs,
    #[serde(rename="tensorflow_saved_model_bundle")]
    TensorflowSavedModelBundle,
    #[serde(rename="torchscript")]
    Torchscript,
}

impl AsPartial for WeightsFormat {
    type Partial = String;
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type")]
pub enum ModelWeightsEnum{
    #[serde(rename = "keras_hdf5")]
    KerasHdf5WeightsDescr(KerasHdf5WeightsDescr),
    #[serde(rename="onnx")]
    OnnxWeightsDescr(OnnxWeightsDescr),
    #[serde(rename="pytorch_state_dict")]
    PytorchStateDictWeightsDescr(PytorchStateDictWeightsDescr),
    #[serde(rename="tensorflow_js")]
    TensorflowJsWeightsDescr(TensorflowJsWeightsDescr),
    #[serde(rename="tensorflow_saved_model_bundle")]
    TensorflowSavedModelBundleWeightsDescr(TensorflowSavedModelBundleWeightsDescr),
    #[serde(rename="torchscript")]
    TorchscriptWeightsDescr(TorchscriptWeightsDescr),
}

impl AsPartial for ModelWeightsEnum {
    type Partial = String;
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, AsPartial)]
#[aspartial(name=PartialWeightsDescrBase)]
pub struct WeightsDescrBase{
    pub source: FileReference,
    #[serde(default)]
    pub sha256: Option<Sha256>,
    #[serde(default)]
    pub authors: Option<Vec<Author2>>,
    pub parent: Option<WeightsFormat>,
}


#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, AsPartial)]
#[aspartial(name = PartialKerasHdf5WeightsDescr)]
pub struct KerasHdf5WeightsDescr{
    #[serde(flatten)]
    pub base: WeightsDescrBase,
    pub tensorflow_version: Version,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[derive(derive_more::Display)]
pub struct OnnxOpsetVersion(u32);

impl AsPartial for OnnxOpsetVersion {
    type Partial = u32;
}

impl TryFrom<u32> for OnnxOpsetVersion{
    type Error = ModelWeightsParsingError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 7{
            Err(ModelWeightsParsingError::BadOnnxOpsetVersion(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl From<OnnxOpsetVersion> for u32{
    fn from(value: OnnxOpsetVersion) -> Self {
        value.0
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone, AsPartial)]
#[aspartial(name = PartialOnnxWeightsDescrBase)]
pub struct OnnxWeightsDescr{
    #[serde(flatten)]
    pub base: WeightsDescrBase,
    pub opset_version: OnnxOpsetVersion,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PyTorchArchitectureFromFileDescr{
    #[serde(flatten)]
    pub file_descr: FileDescription,
    /// Identifier of the callable that returns a torch.nn.Module instance."""
    /// examples: "MyNetworkClass", "get_my_model"
    pub callable: Identifier,
    /// key word arguments for the `callable`
    pub kwargs: serde_json::Map<String, serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PyTorchArchitectureFromLibraryDescr{
    /// Identifier of the callable that returns a torch.nn.Module instance.
    /// examples: "MyNetworkClass", "get_my_model"
    pub callable: Identifier,
    /// key word arguments for the `callable`
    pub kwargs: serde_json::Map<String, serde_json::Value>,
    /// Where to import the callable from, i.e. `from <import_from> import <callable>`
    pub import_from: String
}


#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub enum PytorchArchitectureDescr{
    FromLibraryDescr(PyTorchArchitectureFromLibraryDescr), // must come first because untagged
    FromFileDescr(PyTorchArchitectureFromFileDescr),
}

impl From<PyTorchArchitectureFromLibraryDescr> for PytorchArchitectureDescr{
    fn from(value: PyTorchArchitectureFromLibraryDescr) -> Self {
        Self::FromLibraryDescr(value)
    }
}

impl From<PyTorchArchitectureFromFileDescr> for PytorchArchitectureDescr{
    fn from(value: PyTorchArchitectureFromFileDescr) -> Self {
        Self::FromFileDescr(value)
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct PytorchStateDictWeightsDescr{
    #[serde(flatten)]
    pub base: WeightsDescrBase,
    pub architecture: PytorchArchitectureDescr,
    /// Version of the PyTorch library used.
    /// If `architecture.depencencies` is specified it has to include pytorch and any version pinning has to be compatible.
    pub pytorch_version: Version,
    ///Custom depencies beyond pytorch.
    ///
    ///The conda environment file should include pytorch and any version pinning has to be compatible with
    ///
    ///`pytorch_version`.
    #[serde(default)]
    pub dependencies: Option<FileDescription<EnvironmentFile>>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct TensorflowJsWeightsDescr{
    #[serde(flatten)]
    pub base: WeightsDescrBase,
    /// Version of the TensorFlow library used
    pub tensorflow_version: Version,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct TensorflowSavedModelBundleWeightsDescr{
    #[serde(flatten)]
    pub base: WeightsDescrBase,

    /// Version of the TensorFlow library used
    pub tensorflow_version: Version,
    /// Custom dependencies beyond tensorflow.
    /// Should include tensorflow and any version pinning has to be compatible with `tensorflow_version
    #[serde(default)]
    pub dependencies: Option<FileDescription<EnvironmentFile>>,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct TorchscriptWeightsDescr{
    #[serde(flatten)]
    pub base: WeightsDescrBase,
    /// Version of the PyTorch library used
    pub pytorch_version: Version
}
