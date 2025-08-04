use std::path::PathBuf;
use std::sync::Arc;

use ::aspartial::AsPartial;

use bioimg_runtime::zip_archive_ext::SharedZipArchive;
use bioimg_runtime as rt;
use bioimg_spec::rdf::{self, Author2};
use bioimg_spec::rdf::author::PartialAuthor2;
use bioimg_spec::rdf::cite_entry::{CiteEntry2Msg, PartialCiteEntry, PartialCiteEntry2Msg};
use bioimg_spec::rdf::file_description::PartialFileDescription;
use bioimg_spec::rdf::maintainer::PartialMaintainer;
use bioimg_spec::rdf::model::{self as modelrdf, AxisType};
use crate::widgets::author_widget::AuthorWidget;
use crate::widgets::onnx_weights_widget::OnnxWeightsWidget;
use crate::widgets::posstprocessing_widget::PostprocessingWidget;

use crate::widgets::pytorch_statedict_weights_widget::PytorchStateDictWidget;
use crate::widgets::weights_widget::{KerasHdf5WeightsWidget, TorchscriptWeightsWidget};
use crate::widgets::Restore;

type Partial<T> = <T as AsPartial>::Partial;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AuthorWidgetRawData{
    pub name_widget: String,
    pub affiliation_widget: Option<String>,
    pub email_widget: Option<String>,
    pub github_user_widget: Option<String>,
    pub orcid_widget: Option<String>,
}

impl AuthorWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial_author: Partial<rdf::Author2>) -> Self {
        Self{
            name_widget: partial_author.name.unwrap_or(String::new()),
            affiliation_widget: partial_author.affiliation.unwrap_or(None),
            email_widget: partial_author.email.unwrap_or(None),
            github_user_widget: partial_author.github_user.unwrap_or(None),
            orcid_widget: partial_author.orcid.unwrap_or(None),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct CiteEntryWidgetRawData {
    pub citation_text_widget: String,
    pub doi_widget: Option<String>,
    pub url_widget: Option<String>,
}

impl CiteEntryWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial_cite: PartialCiteEntry2Msg) -> Self {
        Self{
            citation_text_widget: partial_cite.text.unwrap_or(String::new()),
            doi_widget: partial_cite.doi,
            url_widget: partial_cite.url,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct MaintainerWidgetRawData {
    pub github_user_widget: String,
    pub affiliation_widget: Option<String>,
    pub email_widget: Option<String>,
    pub orcid_widget: Option<String>,
    pub name_widget: Option<String>,
}

impl MaintainerWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: PartialMaintainer) -> Self{
        Self {
            github_user_widget: partial.github_user.unwrap_or(String::new()),
            affiliation_widget: partial.affiliation.unwrap_or(None),
            email_widget: partial.email.unwrap_or(None),
            orcid_widget: partial.orcid.unwrap_or(None),
            name_widget: partial.name.unwrap_or(None),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FileWidgetRawData{
    Empty,
    AboutToLoad{path: PathBuf},
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum TestTensorWidgetRawData{
    Empty,
    Loaded{path: Option<PathBuf>, data: Vec<u8>},
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum LocalFileSourceWidgetRawData{
    #[default]
    Empty,
    InMemoryData{name: Option<String>, data: Arc<[u8]>},
    AboutToLoad{path: String, inner_path: Option<String>}
}

impl LocalFileSourceWidgetRawData{
    pub fn from_partial(archive: &SharedZipArchive, partial: Option<String>) -> Self{
        let Some(raw_path) = partial else {
            return Self::Empty
        };
        let zip_entry_path = match archive.identifier(){
            rt::zip_archive_ext::ZipArchiveIdentifier::Path(path) => {
                return Self::AboutToLoad { path: path.to_string_lossy().to_string(), inner_path: Some(raw_path) };
            },
            rt::zip_archive_ext::ZipArchiveIdentifier::Name(name) => name,
        };
        match archive.read_full_entry(&zip_entry_path) {
            Ok(data) => {
                Self::InMemoryData { name: Some(zip_entry_path.clone()), data: Arc::from(data.as_slice()) }
            },
            Err(e) => {
                log::warn!("Could not load contents of {raw_path}/{zip_entry_path}: {e}");
                Self::Empty
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FileSourceWidgetRawData{
    Local(LocalFileSourceWidgetRawData),
    Url(String),
}

impl Default for FileSourceWidgetRawData {
    fn default() -> Self {
        Self::Local(LocalFileSourceWidgetRawData::Empty)
    }
}

impl FileSourceWidgetRawData {
    fn from_partial(archive: &SharedZipArchive, partial: Option<String>) -> Self{
        if let Some(raw) = &partial {
            if let Ok(url) = rdf::HttpUrl::try_from(raw.clone()) { //FIXME: parse?
                return Self::Url(url.to_string())
            };
        };
        Self::Local(LocalFileSourceWidgetRawData::from_partial(archive, partial))
    }
    pub fn from_partial_file_descr(archive: &SharedZipArchive, partial: Option<PartialFileDescription>) -> Self{
        let Some(partial_descr) = partial else {
            return Self::Local(LocalFileSourceWidgetRawData::Empty)
        };
        Self::from_partial(archive, partial_descr.source)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ImageWidget2LoadingStateRawData{
    Empty,
    Forced{img_bytes: Vec<u8>}
}

impl ImageWidget2LoadingStateRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Option<String>) -> Self {
        let Some(entry_path) = partial else {
            return Self::Empty
        };
        match archive.read_full_entry(&entry_path){
            Ok(img_bytes) => Self::Forced { img_bytes },
            Err(e) => {
                log::warn!("Could not load image {}/{entry_path}: {e}", archive.identifier());
                Self::Empty
            }
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct ImageWidget2RawData{
    pub file_source_widget: FileSourceWidgetRawData,
    pub loading_state: ImageWidget2LoadingStateRawData,
}

impl ImageWidget2RawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Option<String>) -> Self {
        let file_source_state = FileSourceWidgetRawData::from_partial(archive, partial.clone());
        Self{
            file_source_widget: file_source_state,
            // FIXME: double check this. I think it's not forced because that'd be smth like a cpy/paste
            // and this is loading from the archive
            loading_state: ImageWidget2LoadingStateRawData::Empty,
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct SpecialImageWidgetRawData{
    pub image_widget: ImageWidget2RawData,
}

impl SpecialImageWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Option<String>) -> Self {
        Self { image_widget: ImageWidget2RawData::from_partial(archive, partial) }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum IconWidgetRawData{
    Emoji(String),
    Image(SpecialImageWidgetRawData),
}

impl IconWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: <rdf::Icon as AsPartial>::Partial) -> Self {
        if archive.has_entry(&partial) {
            let img_data = SpecialImageWidgetRawData::from_partial(archive, Some(partial));
            return Self::Image(img_data);
        }
        Self::Emoji(partial)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CollapsibleWidgetRawData<Inner: Restore>{
    pub is_closed: bool,
    pub inner: Inner::RawData,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct VersionWidgetRawData{
    pub raw: String,
}

impl VersionWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: <rdf::Version as AsPartial>::Partial) -> Self {
        Self{ raw: partial.to_string() }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct CodeEditorWidgetRawData{
    pub raw: String,
}

type JsonMap = serde_json::Map<String, serde_json::Value>;

impl CodeEditorWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: JsonMap) -> Self {
        Self{raw: serde_json::to_string_pretty(&partial).unwrap()}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PhysicalScaleWidgetRawData<T>{
    pub raw_scale: String,
    pub unit_widget: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BatchAxisWidgetRawData{
    pub description_widget: String,
    pub staging_allow_auto_size: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ChannelNamesModeRawData{
    Explicit,
    Pattern,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum AxisSizeModeRawData{
    Fixed,
    Reference,
    Parameterized,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ParameterizedAxisSizeWidgetRawData {
    pub staging_min: usize,
    pub staging_step: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnyAxisSizeWidgetRawData {
    pub mode: AxisSizeModeRawData,

    pub staging_fixed_size: usize,
    pub staging_size_ref: AxisSizeReferenceWidgetRawData,
    pub staging_parameterized: ParameterizedAxisSizeWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IndexAxisWidgetRawData {
    pub description_widget: String,
    pub size_widget: AnyAxisSizeWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AxisSizeReferenceWidgetRawData {
    pub staging_tensor_id: String,
    pub staging_axis_id: String,
    pub staging_offset: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChannelAxisWidgetRawData {
    pub description_widget: String,

    pub channel_names_mode_widget: ChannelNamesModeRawData,
    pub channel_extent_widget: usize,
    pub channel_name_prefix_widget: String,
    pub channel_name_suffix_widget: String,

    pub staging_explicit_names: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputSpaceAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: AnyAxisSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::SpaceUnit>
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputTimeAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: AnyAxisSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::TimeUnit>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputAxisWidgetRawData {
    pub axis_type_widget: bioimg_spec::rdf::model::axes::AxisType,
    pub batch_axis_widget: BatchAxisWidgetRawData,
    pub channel_axis_widget: ChannelAxisWidgetRawData,
    pub index_axis_widget: IndexAxisWidgetRawData,
    pub space_axis_widget: InputSpaceAxisWidgetRawData,
    pub time_axis_widget: InputTimeAxisWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct WeightsDescrBaseWidgetRawData{
    pub source_widget: FileSourceWidgetRawData,
    pub authors_widget: Option<Vec<CollapsibleWidgetRawData<AuthorWidget>>>,
}

impl WeightsDescrBaseWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::WeightsDescrBase as AsPartial>::Partial,
    ) -> Self{
        let source = FileSourceWidgetRawData::from_partial(archive, partial.source);
        let authors = partial.authors.map(|authors| {
            authors.into_iter()
                .map(|author|{
                    let author_state = AuthorWidgetRawData::from_partial(archive, author);
                    CollapsibleWidgetRawData{is_closed: true, inner: author_state}
                })
                .collect::<Vec<_>>()
        });
        Self{source_widget: source, authors_widget: authors}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TorchscriptWeightsWidgetRawData{
    pub base_widget: WeightsDescrBaseWidgetRawData,
    pub pytorch_version_widget: VersionWidgetRawData,
}

impl TorchscriptWeightsWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::TorchscriptWeightsDescr>) -> Self {
        let base = partial.base.map(|partial| WeightsDescrBaseWidgetRawData::from_partial(archive, partial)).unwrap_or_default();
        let version = partial.pytorch_version.map(|partial| VersionWidgetRawData::from_partial(archive, partial)).unwrap_or_default();
        Self{base_widget: base, pytorch_version_widget: version}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct JsonObjectEditorWidgetRawData{
    pub code_editor_widget: CodeEditorWidgetRawData,
}

impl JsonObjectEditorWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: JsonMap) -> Self {
        let code_editor_widget = CodeEditorWidgetRawData::from_partial(archive, partial);
        Self{code_editor_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct CondaEnvEditorWidgetRawData{
    pub code_editor_widget: CodeEditorWidgetRawData,
}

impl CondaEnvEditorWidgetRawData {
    pub fn from_partial_file_descr(archive: &SharedZipArchive, partial: Partial<rdf::EnvironmentFileDescr>) -> Self{
        let Some(source) = partial.source else {
            return Self::default()
        };

        let data = match archive.read_full_entry(&source) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Could not read data from {}/{source}: {e}", archive.identifier());
                return Self::default();
            }
        };
        let data_string = match String::from_utf8(data) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Could not decode data from {}/{source}: {e}", archive.identifier());
                return Self::default();
            }
        };
        Self{ code_editor_widget: CodeEditorWidgetRawData { raw: data_string }}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum PytorchArchModeRawData{
    #[default]
    FromFile,
    FromLib
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PytorchArchWidgetRawData{
    pub mode_widget: PytorchArchModeRawData,
    pub callable_widget: String,
    pub kwargs_widget: JsonObjectEditorWidgetRawData,
    pub import_from_widget: String,
    pub source_widget: FileSourceWidgetRawData,
}

impl PytorchArchWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<rdf::model::PytorchArchitectureDescr>,
    ) -> Self{
        let mut mode_widget = PytorchArchModeRawData::FromFile;
        let mut callable = String::new();
        let mut import_from = String::new();
        let mut kwargs_state = String::new();
        let mut source_widget = FileSourceWidgetRawData::default();

        if let Some(from_file_descr) = partial.from_file_descr {
            callable = from_file_descr.callable.unwrap_or_default();
            if let Some(kwargs) = &from_file_descr.kwargs {
                kwargs_state = serde_json::to_string_pretty(kwargs).unwrap();
            }
            if from_file_descr.file_descr.is_some(){
                mode_widget = PytorchArchModeRawData::FromFile;
            }
            source_widget = FileSourceWidgetRawData::from_partial_file_descr(archive, from_file_descr.file_descr);
        }
        if let Some(from_lib) = partial.from_library_descr {
            if callable.is_empty(){
                callable = from_lib.callable.unwrap_or_default();
            }
            if let Some(imp_from) = from_lib.import_from {
                import_from = imp_from;
                mode_widget = PytorchArchModeRawData::FromLib;
            }
            if let Some(kwargs) = &from_lib.kwargs {
                if kwargs_state.is_empty() {
                    kwargs_state = serde_json::to_string_pretty(kwargs).unwrap();
                }
            }
        }

        Self{
            mode_widget,
            callable_widget: callable,
            kwargs_widget: JsonObjectEditorWidgetRawData {
                code_editor_widget: CodeEditorWidgetRawData { raw: kwargs_state }
            },
            import_from_widget: import_from,
            source_widget,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PytorchStateDictWidgetRawData{
    pub base_widget: WeightsDescrBaseWidgetRawData,
    pub architecture_widget: PytorchArchWidgetRawData,
    pub version_widget: VersionWidgetRawData,
    pub dependencies_widget: Option<CondaEnvEditorWidgetRawData>,
}

impl PytorchStateDictWidgetRawData{
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::PytorchStateDictWeightsDescr as AsPartial>::Partial,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetRawData::from_partial(archive, base))
            .unwrap_or_default();
        let architecture = partial.architecture
            .map(|arch| PytorchArchWidgetRawData::from_partial(archive, arch))
            .unwrap_or_default();
        let version = partial.pytorch_version
            .map(|ver| VersionWidgetRawData::from_partial(archive, ver))
            .unwrap_or_default();
        let dependencies = partial.dependencies
            .map(|file_descr| CondaEnvEditorWidgetRawData::from_partial_file_descr(archive, file_descr));
        Self{base_widget: base, architecture_widget: architecture, version_widget: version, dependencies_widget: dependencies}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OnnxWeightsWidgetRawData{
    pub base_widget: WeightsDescrBaseWidgetRawData,
    pub opset_version_widget: u32,
}

impl OnnxWeightsWidgetRawData{
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::OnnxWeightsDescr as AsPartial>::Partial,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetRawData::from_partial(archive, base))
            .unwrap_or_default();
        let version = partial.opset_version.unwrap_or_default();
        Self{base_widget: base, opset_version_widget: version}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KerasHdf5WeightsWidgetRawData{
    pub base_widget: WeightsDescrBaseWidgetRawData,
    pub tensorflow_version_widget: VersionWidgetRawData,
}

impl KerasHdf5WeightsWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::KerasHdf5WeightsDescr as AsPartial>::Partial,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetRawData::from_partial(archive, base))
            .unwrap_or_default();
        let version = partial.tensorflow_version
            .map(|version| VersionWidgetRawData::from_partial(archive, version))
            .unwrap_or_default();
        Self{base_widget: base, tensorflow_version_widget: version}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct WeightsWidgetRawData{
    pub keras_weights_widget: Option<CollapsibleWidgetRawData<KerasHdf5WeightsWidget>>,
    pub torchscript_weights_widget: Option<CollapsibleWidgetRawData<TorchscriptWeightsWidget>>,
    pub pytorch_state_dict_weights_widget: Option<CollapsibleWidgetRawData<PytorchStateDictWidget>>,
    pub onnx_weights_widget: Option<CollapsibleWidgetRawData<OnnxWeightsWidget>>,
}

impl WeightsWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::WeightsDescr>) -> Self{
        let keras = partial.keras_hdf5.map(|partial| {
            let weights = KerasHdf5WeightsWidgetRawData::from_partial(archive, partial);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        let torchscript = partial.torchscript.map(|partial| {
            let weights = TorchscriptWeightsWidgetRawData::from_partial(archive, partial);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        let pytorch_state_dict = partial.pytorch_state_dict.map(|partial|{
            let weights = PytorchStateDictWidgetRawData::from_partial(archive, partial);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        let onnx = partial.onnx.map(|partial|{
            let weights = OnnxWeightsWidgetRawData::from_partial(archive, partial);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        Self{
            keras_weights_widget: keras,
            torchscript_weights_widget: torchscript,
            pytorch_state_dict_weights_widget: pytorch_state_dict,
            onnx_weights_widget: onnx,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputTensorWidgetRawData {
    pub id_widget: String,
    pub is_optional: bool,
    pub description_widget: String,
    pub axis_widgets: Vec<InputAxisWidgetRawData>,
    pub test_tensor_widget: TestTensorWidgetRawData,
    pub preprocessing_widget: Vec<PreprocessingWidgetRawData>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum PreprocessingWidgetModeRawData {
    Binarize,
    Clip,
    ScaleLinear,
    Sigmoid,
    ZeroMeanUnitVariance,
    ScaleRange,
    EnsureDtype,
    FixedZmuv,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum BinarizeModeRawData{
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SimpleBinarizeWidgetRawData{
    pub threshold_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BinarizeAlongAxisWidgetRawData{
    pub thresholds_widget: Vec<String>,
    pub axis_id_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct BinarizePreprocessingWidgetRawData{
    pub mode: BinarizeModeRawData,
    pub simple_binarize_widget: SimpleBinarizeWidgetRawData,
    pub binarize_along_axis_wiget: BinarizeAlongAxisWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClipWidgetRawData{
    pub min_widget: String,
    pub max_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ScaleLinearModeRawData{
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SimpleScaleLinearWidgetRawData{
    pub gain_widget: String,
    pub offset_widget: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ScaleLinearAlongAxisWidgetRawData{
    pub axis_widget: String,
    pub gain_offsets_widget: Vec<SimpleScaleLinearWidgetRawData>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ZeroMeanUnitVarianceWidgetRawData{
    pub axes_widget: Option<Vec<String>>,
    pub epsilon_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PercentilesWidgetRawData{
    pub min_widget: String,
    pub max_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScaleRangeWidgetRawData{
    pub axes_widget: Option<Vec<String>>,
    pub percentiles_widget: PercentilesWidgetRawData,
    pub epsilon_widget: String,
    pub reference_tensor: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ZmuvWidgetModeRawData{
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SimpleFixedZmuvWidgetRawData{
    pub mean_widget: String,
    pub std_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixedZmuvAlongAxisWidgetRawData{
    pub axis_widget: String,
    pub mean_and_std_widget: Vec<SimpleFixedZmuvWidgetRawData>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixedZmuvWidgetRawData{
    pub mode_widget: ZmuvWidgetModeRawData,
    pub simple_widget: SimpleFixedZmuvWidgetRawData,
    pub along_axis_widget: FixedZmuvAlongAxisWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScaleLinearWidgetRawData{
    pub mode: ScaleLinearModeRawData,
    pub simple_widget: SimpleScaleLinearWidgetRawData,
    pub along_axis_widget: ScaleLinearAlongAxisWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PreprocessingWidgetRawData{
    pub mode: PreprocessingWidgetModeRawData,
    pub binarize_widget: BinarizePreprocessingWidgetRawData,
    pub clip_widget: ClipWidgetRawData,
    pub scale_linear_widget: ScaleLinearWidgetRawData,
    // pub sigmoid sigmoid has no widget since it has no params
    pub zero_mean_unit_variance_widget: ZeroMeanUnitVarianceWidgetRawData,
    pub scale_range_widget: ScaleRangeWidgetRawData,
    pub ensure_dtype_widget: modelrdf::DataType,
    pub fixed_zmuv_widget: FixedZmuvWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputSpacetimeSizeWidgetRawData{
    pub has_halo: bool,
    pub halo_widget: u64,
    pub size_widget: AnyAxisSizeWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputTimeAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: OutputSpacetimeSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::TimeUnit>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputSpaceAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: OutputSpacetimeSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::SpaceUnit>
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputAxisWidgetRawData {
    pub axis_type_widget: AxisType,

    pub batch_axis_widget: BatchAxisWidgetRawData,
    pub channel_axis_widget: ChannelAxisWidgetRawData,
    pub index_axis_widget: IndexAxisWidgetRawData,
    pub space_axis_widget: OutputSpaceAxisWidgetRawData,
    pub time_axis_widget: OutputTimeAxisWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum PostprocessingWidgetModeRawData {
    Binarize,
    Clip,
    ScaleLinear,
    Sigmoid,
    ZeroMeanUnitVariance,
    ScaleRange,
    EnsureDtype,
    FixedZmuv,
    ScaleMeanVariance,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScaleMeanVarianceWidgetRawData{
    pub reference_tensor_widget: String,
    pub axes_widget: Option<Vec<String>>,
    pub eps_widget: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostprocessingWidgetRawData{
    pub mode: PostprocessingWidgetModeRawData,
    pub binarize_widget: BinarizePreprocessingWidgetRawData,
    pub clip_widget: ClipWidgetRawData,
    pub scale_linear_widget: ScaleLinearWidgetRawData,
    // pub sigmoid sigmoid has no widget since it has no params
    pub zero_mean_unit_variance_widget: ZeroMeanUnitVarianceWidgetRawData,
    pub scale_range_widget: ScaleRangeWidgetRawData,
    pub ensure_dtype_widget: modelrdf::DataType,
    pub fixed_zmuv_widget: FixedZmuvWidgetRawData,
    pub scale_mean_var_widget: ScaleMeanVarianceWidgetRawData,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputTensorWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,
    pub axis_widgets: Vec<OutputAxisWidgetRawData>,
    pub test_tensor_widget: TestTensorWidgetRawData,
    pub postprocessing_widgets: Vec<CollapsibleWidgetRawData<PostprocessingWidget>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ModelInterfaceWidgetRawData {
    pub input_widgets: Vec<InputTensorWidgetRawData>,
    pub output_widgets: Vec<OutputTensorWidgetRawData>,
}

#[derive(serde::Serialize, serde::Deserialize, strum::VariantNames)]
#[serde(tag = "app_state_raw_data_version")]
pub enum AppStateRawData{
    Version1(AppState1RawData),
}

#[derive(thiserror::Error, Debug)]
pub enum ProjectLoadError{
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Could not parse BSON: {0}")]
    BsonParsingError(#[from] bson::de::Error),
    #[error("No version in project data")]
    MissingVersion,
    #[error("Could not parse project of version {found_version}")]
    FutureVersion{ found_version: String },
}

impl AppStateRawData{
    pub fn supported_versions() -> &'static [&'static str]{
        <Self as strum::VariantNames>::VARIANTS        
    }

    pub fn highest_supported_version() -> &'static str{
        *Self::supported_versions().last().unwrap()
    }

    pub fn save(&self, writer: impl std::io::Write) -> Result<(), bson::ser::Error>{
        let doc = bson::to_document(self)?;
        doc.to_writer(writer)
    }

    pub fn load(reader: impl std::io::Read) -> Result<Self, ProjectLoadError>{
        let doc: bson::Document = bson::from_reader(reader)?;
        let found_version = match doc.get("app_state_raw_data_version"){
            Some(bson::Bson::String(version)) => version.to_owned(),
            _ => return Err(ProjectLoadError::MissingVersion)
        };
        if Self::supported_versions().iter().find(|ver| **ver == found_version.as_str()).is_none(){
            return Err(ProjectLoadError::FutureVersion { found_version })
        }
        Ok(bson::from_document::<Self>(doc)?)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppState1RawData{
    pub staging_name: String,
    pub staging_description: String,
    pub cover_images: Vec<SpecialImageWidgetRawData>,
    #[serde(default)] // added after AppState1RawData
    pub model_id_widget: Option<String>,
    pub staging_authors: Vec<AuthorWidgetRawData>,
    pub attachments_widget: Vec<FileSourceWidgetRawData>,
    pub staging_citations: Vec<CiteEntryWidgetRawData>,
    #[serde(default)] // added after AppState1RawData
    pub custom_config_widget: Option<JsonObjectEditorWidgetRawData>,
    pub staging_git_repo: Option<String>,
    pub icon_widget: Option<IconWidgetRawData>,
    #[serde(default)] // added after AppState1RawData
    pub links_widget: Vec<String>,
    pub staging_maintainers: Vec<MaintainerWidgetRawData>,
    pub staging_tags: Vec<String>,
    pub staging_version: Option<VersionWidgetRawData>,

    pub staging_documentation: CodeEditorWidgetRawData,
    pub staging_license: ::bioimg_spec::rdf::LicenseId,
    //badges
    pub model_interface_widget: ModelInterfaceWidgetRawData,
    ////
    pub weights_widget: WeightsWidgetRawData,
}
