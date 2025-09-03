//! Types in this module are meant to serve for saving and loading model drafts.
//! Ideally, they should never be changed once a version of the model builder
//! GUI is published, so as not to invalidate models saved by previous versions.
//! New types can be created to account for newer functionality, with conversions
//! between the old and the new implemented to keep backwards compatibility.`
//!
//! These types are usually the `Self::SavedData` associated type in `Restore`
//! impelementations for widges.

use std::path::PathBuf;
use std::sync::Arc;

use ::aspartial::AsPartial;

use bioimg_runtime::zip_archive_ext::SharedZipArchive;
use bioimg_runtime as rt;
use bioimg_spec::rdf;
use bioimg_spec::rdf::cite_entry::PartialCiteEntry2Msg;
use bioimg_spec::rdf::file_description::PartialFileDescription;
use bioimg_spec::rdf::maintainer::PartialMaintainer;
use bioimg_spec::rdf::model::{self as modelrdf, AxisType};
use bioimg_spec::util::PartialSingleOrMultiple;
use crate::widgets::author_widget::AuthorWidget;
use crate::widgets::onnx_weights_widget::OnnxWeightsWidget;
use crate::widgets::posstprocessing_widget::PostprocessingWidget;

use crate::widgets::pytorch_statedict_weights_widget::PytorchStateDictWidget;
use crate::widgets::weights_widget::{KerasHdf5WeightsWidget, TorchscriptWeightsWidget};
use crate::widgets::Restore;

type Partial<T> = <T as AsPartial>::Partial;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AuthorWidgetSavedData{
    pub name_widget: String,
    pub affiliation_widget: Option<String>,
    pub email_widget: Option<String>,
    pub github_user_widget: Option<String>,
    pub orcid_widget: Option<String>,
}

impl AuthorWidgetSavedData {
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
pub struct CiteEntryWidgetSavedData {
    pub citation_text_widget: String,
    pub doi_widget: Option<String>,
    pub url_widget: Option<String>,
}

impl CiteEntryWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial_cite: PartialCiteEntry2Msg) -> Self {
        Self{
            citation_text_widget: partial_cite.text.unwrap_or(String::new()),
            doi_widget: partial_cite.doi,
            url_widget: partial_cite.url,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct MaintainerWidgetSavedData {
    pub github_user_widget: String,
    pub affiliation_widget: Option<String>,
    pub email_widget: Option<String>,
    pub orcid_widget: Option<String>,
    pub name_widget: Option<String>,
}

impl MaintainerWidgetSavedData {
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

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum FileWidgetSavedData{
    #[default]
    Empty,
    AboutToLoad{path: PathBuf},
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum TestTensorWidgetSavedData{
    #[default]
    Empty,
    Loaded{path: Option<PathBuf>, data: Vec<u8>},
}

impl TestTensorWidgetSavedData {
    pub fn from_partial<W: std::fmt::Write>(
        archive: &SharedZipArchive,
        partial: Partial<rdf::FileDescription>,
        mut warnings: W
    ) -> Self {
        let Some(source) = partial.source else {
            return Self::Empty
        };
        let data = match archive.read_full_entry(&source){
            Ok(bytes) => bytes,
            Err(e) => {
                _ = writeln!(warnings, "Could not read test tensor bytes at '{source}': {e}");
                return Self::Empty;
            }
        };
        Self::Loaded{path: Some(source.into()), data}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum LocalFileSourceWidgetSavedData{
    #[default]
    Empty,
    InMemoryData{name: Option<String>, data: Arc<[u8]>},
    AboutToLoad{path: String, inner_path: Option<String>}
}

impl LocalFileSourceWidgetSavedData{
    pub fn from_partial<W: std::fmt::Write>(archive: &SharedZipArchive, raw_path: String, warnings: &mut W) -> Self{
        let zip_entry_path = match archive.identifier(){
            rt::zip_archive_ext::ZipArchiveIdentifier::Path(path) => {
                return Self::AboutToLoad { path: path.to_string_lossy().to_string(), inner_path: Some(raw_path) };
            },
            rt::zip_archive_ext::ZipArchiveIdentifier::Name(name) => name,
        };
        println!("Gonna aread full entry for {raw_path}");
        match archive.read_full_entry(&raw_path) {
            Ok(data) => {
                Self::InMemoryData { name: Some(zip_entry_path.clone()), data: Arc::from(data.as_slice()) }
            },
            Err(e) => {
                _ = writeln!(warnings, "Could not load contents of {zip_entry_path}/{raw_path}: {e}");
                Self::Empty
            },
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum FileSourceWidgetSavedData{
    Local(LocalFileSourceWidgetSavedData),
    Url(String),
}

impl Default for FileSourceWidgetSavedData {
    fn default() -> Self {
        Self::Local(LocalFileSourceWidgetSavedData::Empty)
    }
}

impl FileSourceWidgetSavedData {
    fn from_partial<W: std::fmt::Write>(archive: &SharedZipArchive, partial: String, warnings: &mut W) -> Self{
        if let Ok(url) = rdf::HttpUrl::try_from(partial.clone()) { //FIXME: parse?
            return Self::Url(url.to_string())
        };
        Self::Local(LocalFileSourceWidgetSavedData::from_partial(archive, partial, warnings))
    }
    pub fn from_partial_file_descr(
        archive: &SharedZipArchive,
        partial: PartialFileDescription,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let Some(source) = partial.source else {
            return Default::default()
        };
        Self::from_partial(archive, source, warnings)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ImageWidget2LoadingStateSavedData{
    Empty,
    Forced{img_bytes: Vec<u8>}
}

// impl ImageWidget2LoadingStateSavedData {
//     pub fn from_partial(archive: &SharedZipArchive, partial: Option<String>) -> Self {
//         let Some(entry_path) = partial else {
//             return Self::Empty
//         };
//         match archive.read_full_entry(&entry_path){
//             Ok(img_bytes) => Self::Forced { img_bytes },
//             Err(e) => {
//                 log::warn!("Could not load image {}/{entry_path}: {e}", archive.identifier());
//                 Self::Empty
//             }
//         }
//     }
// }


#[derive(serde::Serialize, serde::Deserialize)]
pub struct ImageWidget2SavedData{
    pub file_source_widget: FileSourceWidgetSavedData,
    pub loading_state: ImageWidget2LoadingStateSavedData,
}

impl ImageWidget2SavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: String, warnings: &mut impl std::fmt::Write) -> Self {
        let file_source_state = FileSourceWidgetSavedData::from_partial(archive, partial.clone(), warnings);
        Self{
            file_source_widget: file_source_state,
            // FIXME: double check this. I think it's not forced because that'd be smth like a cpy/paste
            // and this is loading from the archive
            loading_state: ImageWidget2LoadingStateSavedData::Empty,
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct SpecialImageWidgetSavedData{
    pub image_widget: ImageWidget2SavedData,
}

impl SpecialImageWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: String,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        Self { image_widget: ImageWidget2SavedData::from_partial(archive, partial, warnings) }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum IconWidgetSavedData{
    Emoji(String),
    Image(SpecialImageWidgetSavedData),
}

impl IconWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::Icon as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        if archive.has_entry(&partial) {
            let img_data = SpecialImageWidgetSavedData::from_partial(archive, partial, warnings);
            return Self::Image(img_data);
        }
        Self::Emoji(partial)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CollapsibleWidgetSavedData<Inner: Restore>{
    pub is_closed: bool,
    pub inner: Inner::SavedData,
}

impl<Inner: Restore> CollapsibleWidgetSavedData<Inner> {
    pub fn new(inner: Inner::SavedData) -> Self {
        Self{inner, is_closed: true}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct VersionWidgetSavedData{
    pub raw: String,
}

impl VersionWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: <rdf::Version as AsPartial>::Partial) -> Self {
        Self{ raw: partial.to_string() }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct CodeEditorWidgetSavedData{
    pub raw: String,
}

type JsonMap = serde_json::Map<String, serde_json::Value>;

impl CodeEditorWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: JsonMap) -> Self {
        Self{raw: serde_json::to_string_pretty(&partial).unwrap()}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PhysicalScaleWidgetSavedData<T>{
    pub raw_scale: String,
    pub unit_widget: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct BatchAxisWidgetSavedData{
    pub description_widget: String,
    pub staging_allow_auto_size: bool,
}

impl BatchAxisWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::BatchAxis>) -> Self{
        Self{
            description_widget: partial.description.to_string(),
            staging_allow_auto_size: partial.size.is_some(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum ChannelNamesModeSavedData{
    #[default]
    Explicit,
    Pattern,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum AxisSizeModeSavedData{
    #[default]
    Fixed,
    Reference,
    Parameterized,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ParameterizedAxisSizeWidgetSavedData {
    pub staging_min: usize,
    pub staging_step: usize,
}

impl ParameterizedAxisSizeWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::ParameterizedAxisSize>) -> Self {
        Self{
            staging_min: partial.min.map(|min| usize::from(min)).unwrap_or(0),
            staging_step: partial.step.map(|step| usize::from(step)).unwrap_or(0),
        }
    } 
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AnyAxisSizeWidgetSavedData {
    pub mode: AxisSizeModeSavedData,

    pub staging_fixed_size: usize,
    pub staging_size_ref: AxisSizeReferenceWidgetSavedData,
    pub staging_parameterized: ParameterizedAxisSizeWidgetSavedData,
}

impl AnyAxisSizeWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::AnyAxisSize>) -> Self {
        let mut mode = AxisSizeModeSavedData::Fixed;
        let mut staging_fixed_size = 0usize;
        let mut staging_size_ref = AxisSizeReferenceWidgetSavedData::default();
        let mut staging_parameterized = ParameterizedAxisSizeWidgetSavedData::default();
        
        if let Some(fixed) = partial.fixed {
            mode = AxisSizeModeSavedData::Fixed;
            staging_fixed_size = fixed.into();
        }
        if let Some(refer) = partial.reference {
            if refer.qualified_axis_id.is_some(){
                mode = AxisSizeModeSavedData::Reference;
            }
            staging_size_ref = AxisSizeReferenceWidgetSavedData::from_partial(archive, refer);
        }
        if let Some(params) = partial.parameterized {
            if params.step.is_some() || params.step.is_some(){
                mode = AxisSizeModeSavedData::Parameterized;
            }
            staging_parameterized = ParameterizedAxisSizeWidgetSavedData::from_partial(archive, params);
        }
        Self{
            mode,
            staging_fixed_size,
            staging_size_ref,
            staging_parameterized
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct IndexAxisWidgetSavedData {
    pub description_widget: String,
    pub size_widget: AnyAxisSizeWidgetSavedData,
}

impl IndexAxisWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::IndexAxis>) -> Self {
        Self{
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|partial| AnyAxisSizeWidgetSavedData::from_partial(archive, partial))
                .unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AxisSizeReferenceWidgetSavedData {
    pub staging_tensor_id: String,
    pub staging_axis_id: String,
    pub staging_offset: usize,
}

impl AxisSizeReferenceWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::AxisSizeReference>) -> Self {
        let tensor_id: String;
        let axis_id: String;

        if let Some(qual_id) = partial.qualified_axis_id{
            tensor_id = match qual_id.tensor_id {
                Some(t_id) => t_id.to_string(),
                None => String::new(),
            };
            axis_id = match qual_id.axis_id{
                Some(a_id) => a_id.to_string(),
                None => String::new(),
            };
        } else {
            tensor_id = String::new();
            axis_id = String::new();
        }
        Self{
            staging_tensor_id: tensor_id,
            staging_axis_id: axis_id,
            staging_offset: partial.offset,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ChannelAxisWidgetSavedData {
    pub description_widget: String,

    pub channel_names_mode_widget: ChannelNamesModeSavedData,
    pub channel_extent_widget: usize,
    pub channel_name_prefix_widget: String,
    pub channel_name_suffix_widget: String,

    pub staging_explicit_names: Vec<String>,
}

impl ChannelAxisWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::ChannelAxis>) -> Self {
        Self{
            description_widget: partial.description.to_string(),
            channel_extent_widget: partial.channel_names.as_ref().map(|names| names.len()).unwrap_or_default(),
            channel_names_mode_widget: ChannelNamesModeSavedData::Explicit,
            staging_explicit_names: match partial.channel_names {
                Some(names) => names.into_iter().map(|name| name.to_string()).collect(),
                None => vec![]
            },
            ..Default::default()
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct InputSpaceAxisWidgetSavedData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: AnyAxisSizeWidgetSavedData,
    pub physical_scale_widget: PhysicalScaleWidgetSavedData<modelrdf::SpaceUnit>
}

impl InputSpaceAxisWidgetSavedData {
    pub fn from_partial<W: std::fmt::Write>(archive: &SharedZipArchive, partial: Partial<rdf::model::SpaceInputAxis>, warnings: &mut W) -> Self{
        Self{
            id_widget: partial.id.to_string(),
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|size| AnyAxisSizeWidgetSavedData::from_partial(archive, size))
                .unwrap_or_default(),
            physical_scale_widget: PhysicalScaleWidgetSavedData {
                raw_scale: partial.scale.to_string(),
                unit_widget: 'unit: {
                    let Some(unit) = partial.unit else {
                        break 'unit None;
                    };
                    let parsed_unit = match unit.parse::<modelrdf::SpaceUnit>() {
                        Ok(parsed_unit) => parsed_unit,
                        Err(e) => {
                            _ = writeln!(warnings, "Could not parse spacial unit '{unit}': {e}");
                            break 'unit None
                        },
                    };
                    Some(parsed_unit)
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct InputTimeAxisWidgetSavedData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: AnyAxisSizeWidgetSavedData,
    pub physical_scale_widget: PhysicalScaleWidgetSavedData<modelrdf::TimeUnit>,
}

impl InputTimeAxisWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::TimeInputAxis>) -> Self{
        Self{
            id_widget: partial.id.to_string(),
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|size| AnyAxisSizeWidgetSavedData::from_partial(archive, size))
                .unwrap_or_default(),
            physical_scale_widget: PhysicalScaleWidgetSavedData {
                raw_scale: partial.scale.to_string(),
                unit_widget: 'unit: {
                    let Some(unit) = partial.unit else {
                        break 'unit None;
                    };
                    let parsed_unit = match unit.parse::<modelrdf::TimeUnit>() {
                        Ok(parsed_unit) => parsed_unit,
                        Err(e) => {
                            log::warn!("Could not parse time unit '{unit}': {e}");
                            break 'unit None
                        },
                    };
                    Some(parsed_unit)
                }
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputAxisWidgetSavedData {
    pub axis_type_widget: bioimg_spec::rdf::model::axes::AxisType,
    pub batch_axis_widget: BatchAxisWidgetSavedData,
    pub channel_axis_widget: ChannelAxisWidgetSavedData,
    pub index_axis_widget: IndexAxisWidgetSavedData,
    pub space_axis_widget: InputSpaceAxisWidgetSavedData,
    pub time_axis_widget: InputTimeAxisWidgetSavedData,
}

impl InputAxisWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::InputAxis>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let mut axis_type_widget = modelrdf::axes::AxisType::Space;
        let batch_axis_widget = match partial.batch {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Batch;
                BatchAxisWidgetSavedData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let channel_axis_widget = match partial.channel{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Channel;
                ChannelAxisWidgetSavedData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let index_axis_widget = match partial.index {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Index;
                IndexAxisWidgetSavedData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let space_axis_widget = match partial.space{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Space;
                InputSpaceAxisWidgetSavedData::from_partial(archive, partial, warnings)
            }
           None => Default::default(),
        };
        let time_axis_widget = match partial.time {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Time;
                InputTimeAxisWidgetSavedData::from_partial(archive, partial)
            }
           None => Default::default(),
        };
        Self{
            axis_type_widget,
            batch_axis_widget,
            channel_axis_widget,
            index_axis_widget,
            space_axis_widget,
            time_axis_widget,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct WeightsDescrBaseWidgetSavedData{
    pub source_widget: FileSourceWidgetSavedData,
    pub authors_widget: Option<Vec<CollapsibleWidgetSavedData<AuthorWidget>>>,
}

impl WeightsDescrBaseWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::WeightsDescrBase as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let source = partial.source
            .map(|src| FileSourceWidgetSavedData::from_partial(archive, src, warnings))
            .unwrap_or_default();
        let authors = partial.authors.map(|authors| {
            authors.into_iter()
                .map(|author|{
                    let author_state = AuthorWidgetSavedData::from_partial(archive, author);
                    CollapsibleWidgetSavedData{is_closed: true, inner: author_state}
                })
                .collect::<Vec<_>>()
        });
        Self{source_widget: source, authors_widget: authors}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TorchscriptWeightsWidgetSavedData{
    pub base_widget: WeightsDescrBaseWidgetSavedData,
    pub pytorch_version_widget: VersionWidgetSavedData,
}

impl TorchscriptWeightsWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<rdf::model::TorchscriptWeightsDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let base = partial.base.map(|partial| WeightsDescrBaseWidgetSavedData::from_partial(archive, partial, warnings)).unwrap_or_default();
        let version = partial.pytorch_version.map(|partial| VersionWidgetSavedData::from_partial(archive, partial)).unwrap_or_default();
        Self{base_widget: base, pytorch_version_widget: version}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct JsonObjectEditorWidgetSavedData{
    pub code_editor_widget: CodeEditorWidgetSavedData,
}

impl JsonObjectEditorWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: JsonMap) -> Self {
        let code_editor_widget = CodeEditorWidgetSavedData::from_partial(archive, partial);
        Self{code_editor_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct CondaEnvEditorWidgetSavedData{
    pub code_editor_widget: CodeEditorWidgetSavedData,
}

impl CondaEnvEditorWidgetSavedData {
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
        Self{ code_editor_widget: CodeEditorWidgetSavedData { raw: data_string }}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum PytorchArchModeSavedData{
    #[default]
    FromFile,
    FromLib
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PytorchArchWidgetSavedData{
    pub mode_widget: PytorchArchModeSavedData,
    pub callable_widget: String,
    pub kwargs_widget: JsonObjectEditorWidgetSavedData,
    pub import_from_widget: String,
    pub source_widget: FileSourceWidgetSavedData,
}

impl PytorchArchWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<rdf::model::PytorchArchitectureDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let mut mode_widget = PytorchArchModeSavedData::FromFile;
        let mut callable = String::new();
        let mut import_from = String::new();
        let mut kwargs_state = String::new();
        let mut source_widget = FileSourceWidgetSavedData::default();

        if let Some(from_file_descr) = partial.from_file_descr {
            callable = from_file_descr.callable.unwrap_or_default();
            if let Some(kwargs) = &from_file_descr.kwargs {
                kwargs_state = serde_json::to_string_pretty(kwargs).unwrap();
            }
            if from_file_descr.file_descr.is_some(){
                mode_widget = PytorchArchModeSavedData::FromFile;
            }
            source_widget = from_file_descr.file_descr
                .map(|fd| {
                    let Some(src) = fd.source else {
                        return Default::default();
                    };
                    FileSourceWidgetSavedData::from_partial(archive, src, warnings)
                })
                .unwrap_or_default();
        }
        if let Some(from_lib) = partial.from_library_descr {
            if callable.is_empty(){
                callable = from_lib.callable.unwrap_or_default();
            }
            if let Some(imp_from) = from_lib.import_from {
                import_from = imp_from;
                mode_widget = PytorchArchModeSavedData::FromLib;
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
            kwargs_widget: JsonObjectEditorWidgetSavedData {
                code_editor_widget: CodeEditorWidgetSavedData { raw: kwargs_state }
            },
            import_from_widget: import_from,
            source_widget,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PytorchStateDictWidgetSavedData{
    pub base_widget: WeightsDescrBaseWidgetSavedData,
    pub architecture_widget: PytorchArchWidgetSavedData,
    pub version_widget: VersionWidgetSavedData,
    pub dependencies_widget: Option<CondaEnvEditorWidgetSavedData>,
}

impl PytorchStateDictWidgetSavedData{
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::PytorchStateDictWeightsDescr as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetSavedData::from_partial(archive, base, warnings))
            .unwrap_or_default();
        let architecture = partial.architecture
            .map(|arch| PytorchArchWidgetSavedData::from_partial(archive, arch, warnings))
            .unwrap_or_default();
        let version = partial.pytorch_version
            .map(|ver| VersionWidgetSavedData::from_partial(archive, ver))
            .unwrap_or_default();
        let dependencies = partial.dependencies
            .map(|file_descr| CondaEnvEditorWidgetSavedData::from_partial_file_descr(archive, file_descr));
        Self{base_widget: base, architecture_widget: architecture, version_widget: version, dependencies_widget: dependencies}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OnnxWeightsWidgetSavedData{
    pub base_widget: WeightsDescrBaseWidgetSavedData,
    pub opset_version_widget: u32,
}

impl OnnxWeightsWidgetSavedData{
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::OnnxWeightsDescr as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetSavedData::from_partial(archive, base, warnings))
            .unwrap_or_default();
        let version = partial.opset_version.unwrap_or_default();
        Self{base_widget: base, opset_version_widget: version}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct KerasHdf5WeightsWidgetSavedData{
    pub base_widget: WeightsDescrBaseWidgetSavedData,
    pub tensorflow_version_widget: VersionWidgetSavedData,
}

impl KerasHdf5WeightsWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::KerasHdf5WeightsDescr as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetSavedData::from_partial(archive, base, warnings))
            .unwrap_or_default();
        let version = partial.tensorflow_version
            .map(|version| VersionWidgetSavedData::from_partial(archive, version))
            .unwrap_or_default();
        Self{base_widget: base, tensorflow_version_widget: version}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct WeightsWidgetSavedData{
    pub keras_weights_widget: Option<CollapsibleWidgetSavedData<KerasHdf5WeightsWidget>>,
    pub torchscript_weights_widget: Option<CollapsibleWidgetSavedData<TorchscriptWeightsWidget>>,
    pub pytorch_state_dict_weights_widget: Option<CollapsibleWidgetSavedData<PytorchStateDictWidget>>,
    pub onnx_weights_widget: Option<CollapsibleWidgetSavedData<OnnxWeightsWidget>>,
}

impl WeightsWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<rdf::model::WeightsDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let keras = partial.keras_hdf5.map(|partial| {
            let weights = KerasHdf5WeightsWidgetSavedData::from_partial(archive, partial, warnings);
            CollapsibleWidgetSavedData{is_closed: true, inner: weights}
        });
        let torchscript = partial.torchscript.map(|partial| {
            let weights = TorchscriptWeightsWidgetSavedData::from_partial(archive, partial, warnings);
            CollapsibleWidgetSavedData{is_closed: true, inner: weights}
        });
        let pytorch_state_dict = partial.pytorch_state_dict.map(|partial|{
            let weights = PytorchStateDictWidgetSavedData::from_partial(archive, partial, warnings);
            CollapsibleWidgetSavedData{is_closed: true, inner: weights}
        });
        let onnx = partial.onnx.map(|partial|{
            let weights = OnnxWeightsWidgetSavedData::from_partial(archive, partial, warnings);
            CollapsibleWidgetSavedData{is_closed: true, inner: weights}
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
pub struct InputTensorWidgetSavedData {
    pub id_widget: String,
    pub is_optional: bool,
    pub description_widget: String,
    pub axis_widgets: Vec<InputAxisWidgetSavedData>,
    pub test_tensor_widget: TestTensorWidgetSavedData,
    pub preprocessing_widget: Vec<PreprocessingWidgetSavedData>,
}

impl InputTensorWidgetSavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::InputTensorDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let mut id_widget = String::new();
        let mut is_optional = false;
        let mut description_widget = String::new();
        let mut axis_widgets = Vec::<InputAxisWidgetSavedData>::new();
        let mut preprocessing_widget = Vec::<PreprocessingWidgetSavedData>::new();
        
        if let Some(meta) = partial.meta {
            if let Some(id) = meta.id {
                id_widget = id;
            }
            is_optional = meta.optional;
            if let Some(description) = meta.description {
                description_widget = description;
            }
            if let Some(axes) = meta.axes {
                for partial_axis in axes{
                    axis_widgets.push(InputAxisWidgetSavedData::from_partial(archive, partial_axis, warnings));
                }
            }
            if let Some(preprocs) = meta.preprocessing {
                for partial_preproc in preprocs {
                    preprocessing_widget.push(PreprocessingWidgetSavedData::from_partial(archive, partial_preproc));
                }
            }
        }
        let test_tensor_widget = partial.test_tensor
            .map(|tt| TestTensorWidgetSavedData::from_partial(archive, tt, warnings))
            .unwrap_or_default();

        Self{id_widget, is_optional, description_widget, axis_widgets, test_tensor_widget, preprocessing_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum PreprocessingWidgetModeSavedData {
    #[default]
    Binarize,
    Clip,
    ScaleLinear,
    Sigmoid,
    ZeroMeanUnitVariance,
    ScaleRange,
    EnsureDtype,
    FixedZmuv,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum BinarizeModeSavedData{
    #[default]
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SimpleBinarizeWidgetSavedData{
    pub threshold_widget: String,
}

impl SimpleBinarizeWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::SimpleBinarizeDescr>) -> Self {
        Self{ threshold_widget: partial.threshold.map(|t| t.to_string()).unwrap_or_default() }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct BinarizeAlongAxisWidgetSavedData{
    pub thresholds_widget: Vec<String>,
    pub axis_id_widget: String,
}

impl BinarizeAlongAxisWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::BinarizeAlongAxisDescr>) -> Self {
        Self{
            thresholds_widget: partial.threshold
                .map(|trs| trs.into_iter().map(|t| t.to_string()).collect())
                .unwrap_or(vec![]),
            axis_id_widget: partial.axis.map(|a| a.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct BinarizePreprocessingWidgetSavedData{
    pub mode: BinarizeModeSavedData,
    pub simple_binarize_widget: SimpleBinarizeWidgetSavedData,
    pub binarize_along_axis_wiget: BinarizeAlongAxisWidgetSavedData,
}

impl BinarizePreprocessingWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::BinarizeDescr>) -> Self {
        let mut mode = BinarizeModeSavedData::Simple;
        let simple_binarize_widget = match partial.simple {
            Some(simple) => {
                mode = BinarizeModeSavedData::Simple;
                SimpleBinarizeWidgetSavedData::from_partial(archive, simple)
            },
            None => SimpleBinarizeWidgetSavedData::default(),
        };
        let binarize_along_axis_wiget = match partial.along_axis {
            Some(along_axis) => {
                mode = BinarizeModeSavedData::AlongAxis;
                BinarizeAlongAxisWidgetSavedData::from_partial(archive, along_axis)
            },
            None => BinarizeAlongAxisWidgetSavedData::default(),
        };
        Self{mode, simple_binarize_widget, binarize_along_axis_wiget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ClipWidgetSavedData{
    pub min_widget: String,
    pub max_widget: String,
}

impl ClipWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ClipDescr>) -> Self {
        Self{
            min_widget: partial.min.map(|min| min.to_string()).unwrap_or_default(),
            max_widget: partial.max.map(|max| max.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum ScaleLinearModeSavedData{
    #[default]
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SimpleScaleLinearWidgetSavedData{
    pub gain_widget: String,
    pub offset_widget: String,
}

impl SimpleScaleLinearWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::SimpleScaleLinearDescr>) -> Self {
        Self{
            gain_widget: partial.gain.to_string(),
            offset_widget: partial.offset.to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct ScaleLinearAlongAxisWidgetSavedData{
    pub axis_widget: String,
    pub gain_offsets_widget: Vec<SimpleScaleLinearWidgetSavedData>,
}

impl ScaleLinearAlongAxisWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleLinearAlongAxisDescr>) -> Self {
        let mut gains = match partial.gain {
            PartialSingleOrMultiple::Single(item) => vec![item],
            PartialSingleOrMultiple::Multiple(items) => items,
        }.into_iter();
        let mut offsets = match partial.offset {
            PartialSingleOrMultiple::Single(item) => vec![item],
            PartialSingleOrMultiple::Multiple(items) => items,
        }.into_iter();

        let mut gain_offsets_widget = Vec::<SimpleScaleLinearWidgetSavedData>::new();
        loop {
            let gain = gains.next();
            let offset = offsets.next();
            if gain.is_none() && offset.is_none() {
                break
            }
            gain_offsets_widget.push(
                SimpleScaleLinearWidgetSavedData {
                    gain_widget: gain.map(|g| g.to_string()).unwrap_or_default(),
                    offset_widget: offset.map(|o| o.to_string()).unwrap_or_default(),
                }
            )
        }

        Self{
            axis_widget: partial.axis.map(|a| a.to_string()).unwrap_or_default(),
            gain_offsets_widget,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ZeroMeanUnitVarianceWidgetSavedData{
    pub axes_widget: Option<Vec<String>>,
    pub epsilon_widget: String,
}

impl ZeroMeanUnitVarianceWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::Zmuv>) -> Self {
        let axes = match partial.axes {
            Some(Some(axes)) => match axes.len() {
                0 => None,
                _ => Some(axes),
            },
            _ => None,
        };
        Self{
            axes_widget: axes, //FIXME: none if empty vec?
            epsilon_widget: partial.eps.to_string(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PercentilesWidgetSavedData{
    pub min_widget: String,
    pub max_widget: String,
}

impl PercentilesWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleRangePercentile>) -> Self {
        Self{
            min_widget: partial.min_percentile.map(|v| v.to_string()).unwrap_or_default(),
            max_widget: partial.max_percentile.map(|v| v.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ScaleRangeWidgetSavedData{
    pub axes_widget: Option<Vec<String>>,
    pub percentiles_widget: PercentilesWidgetSavedData,
    pub epsilon_widget: String,
    pub reference_tensor: Option<String>,
}

impl ScaleRangeWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleRangeDescr>) -> Self {
        let axes = match partial.axes {
            Some(Some(axes)) => match axes.len() {
                0 => None,
                _ => Some(axes),
            },
            _ => None,
        };
        let percentiles_widget = partial.percentiles
            .map(|per| PercentilesWidgetSavedData::from_partial(archive, per))
            .unwrap_or_default();
        Self{
            axes_widget: axes,
            percentiles_widget,
            epsilon_widget: partial.eps.to_string(),
            reference_tensor: partial.reference_tensor.map(|s| s.to_string()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum ZmuvWidgetModeSavedData{
    #[default]
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SimpleFixedZmuvWidgetSavedData{
    pub mean_widget: String,
    pub std_widget: String,
}

impl SimpleFixedZmuvWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::SimpleFixedZmuv>) -> Self {
        Self{
            mean_widget: partial.mean.map(|v| v.to_string()).unwrap_or_default(),
            std_widget: partial.std.map(|v| v.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct FixedZmuvAlongAxisWidgetSavedData{
    pub axis_widget: String,
    pub mean_and_std_widget: Vec<SimpleFixedZmuvWidgetSavedData>,
}

impl FixedZmuvAlongAxisWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::FixedZmuvAlongAxis>) -> Self {
        let mut means = partial.mean.unwrap_or_default().into_iter();
        let mut stds = partial.std.unwrap_or_default().into_iter();

        let mut mean_and_std_widget = Vec::<SimpleFixedZmuvWidgetSavedData >::new();
        loop {
            let mean = means.next();
            let std = stds.next();
            if mean.is_none() && std.is_none() {
                break
            }
            mean_and_std_widget.push(
                SimpleFixedZmuvWidgetSavedData {
                    mean_widget: mean.map(|g| g.to_string()).unwrap_or_default(),
                    std_widget: std.map(|o| o.to_string()).unwrap_or_default(),
                }
            )
        }
        Self {
            axis_widget: partial.axis.map(|v| v.to_string()).unwrap_or_default(),
            mean_and_std_widget,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct FixedZmuvWidgetSavedData{
    pub mode_widget: ZmuvWidgetModeSavedData,
    pub simple_widget: SimpleFixedZmuvWidgetSavedData,
    pub along_axis_widget: FixedZmuvAlongAxisWidgetSavedData,
}

impl FixedZmuvWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::FixedZmuv>) -> Self {
        let mut mode_widget = ZmuvWidgetModeSavedData::Simple;
        let simple_widget = partial.simple
            .map(|partial_preproc| {
                mode_widget = ZmuvWidgetModeSavedData::Simple;
                SimpleFixedZmuvWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let along_axis_widget = partial.along_axis
            .map(|partial_preproc| {
                mode_widget = ZmuvWidgetModeSavedData::AlongAxis;
                FixedZmuvAlongAxisWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        Self{mode_widget, simple_widget, along_axis_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ScaleLinearWidgetSavedData{
    pub mode: ScaleLinearModeSavedData,
    pub simple_widget: SimpleScaleLinearWidgetSavedData,
    pub along_axis_widget: ScaleLinearAlongAxisWidgetSavedData,
}

impl ScaleLinearWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleLinearDescr>) -> Self {
        let mut mode = ScaleLinearModeSavedData::Simple;
        let simple_widget = partial.simple
            .map(|partial_preproc| {
                mode = ScaleLinearModeSavedData::Simple;
                SimpleScaleLinearWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let along_axis_widget = partial.along_axis
            .map(|partial_preproc| {
                mode = ScaleLinearModeSavedData::AlongAxis;
                ScaleLinearAlongAxisWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        Self{mode, simple_widget, along_axis_widget}
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct PreprocessingWidgetSavedData{
    pub mode: PreprocessingWidgetModeSavedData,
    pub binarize_widget: BinarizePreprocessingWidgetSavedData,
    pub clip_widget: ClipWidgetSavedData,
    pub scale_linear_widget: ScaleLinearWidgetSavedData,
    // pub sigmoid sigmoid has no widget since it has no params
    pub zero_mean_unit_variance_widget: ZeroMeanUnitVarianceWidgetSavedData,
    pub scale_range_widget: ScaleRangeWidgetSavedData,
    pub ensure_dtype_widget: modelrdf::DataType,
    pub fixed_zmuv_widget: FixedZmuvWidgetSavedData,
}

impl PreprocessingWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::PreprocessingDescr>) -> Self{
        let mut mode = PreprocessingWidgetModeSavedData::default();
        let binarize_widget = partial.binarize
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeSavedData::Binarize;
                BinarizePreprocessingWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let clip_widget = partial.clip
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeSavedData::Clip;
                ClipWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_linear_widget = partial.scale_linear
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeSavedData::ScaleLinear;
                ScaleLinearWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let zero_mean_unit_variance_widget = partial.zero_mean_unit_variance
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeSavedData::ZeroMeanUnitVariance;
                ZeroMeanUnitVarianceWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_range_widget = partial.scale_range
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeSavedData::ScaleRange;
                ScaleRangeWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let ensure_dtype_widget = partial.ensure_dtype
            .map(|partial_preproc| { 'dtype: {
                mode = PreprocessingWidgetModeSavedData::EnsureDtype;
                let Some(raw_type) = partial_preproc.dtype else{
                    break 'dtype modelrdf::DataType::Float32
                };
                match raw_type.parse::<modelrdf::DataType>() {
                    Ok(dtype) => dtype,
                    Err(e) => {
                        log::warn!("Could not parse dtype '{}': {e}", raw_type);
                        modelrdf::DataType::Float32
                    }
                }
            } })
            .unwrap_or(modelrdf::DataType::Float32);

        let fixed_zmuv_widget = partial.fixed_zero_mean_unit_variance
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeSavedData::FixedZmuv;
                FixedZmuvWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        Self{
            mode,
            binarize_widget,
            clip_widget,
            scale_linear_widget,
            zero_mean_unit_variance_widget,
            scale_range_widget,
            ensure_dtype_widget,
            fixed_zmuv_widget,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct OutputSpacetimeSizeWidgetSavedData{
    pub has_halo: bool,
    pub halo_widget: u64,
    pub size_widget: AnyAxisSizeWidgetSavedData,
}

impl OutputSpacetimeSizeWidgetSavedData {
    fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::axes::output_axes::OutputSpacetimeSize>) -> Self {
        let mut has_halo = false;
        let mut halo_widget = 0u64;
        let mut size_widget = AnyAxisSizeWidgetSavedData::default();

        if let Some(haloed) = partial.haloed {
            has_halo = true;
            if let Some(halo) = haloed.halo {
                halo_widget = halo;
            }
            if let Some(size) = haloed.size {
                if let Some(fixed) = size.fixed{
                    size_widget.staging_fixed_size = fixed.into();
                }
                if let Some(refer) = size.reference {
                    size_widget.staging_size_ref = AxisSizeReferenceWidgetSavedData::from_partial(archive, refer);
                }
            }
        }
        if let Some(standard) = partial.standard {
            if let Some(size) = standard.size{
                //FIXME: detect setting size multiple times?
                size_widget = AnyAxisSizeWidgetSavedData::from_partial(archive, size);
            }
        }

        Self{has_halo, halo_widget, size_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct OutputTimeAxisWidgetSavedData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: OutputSpacetimeSizeWidgetSavedData,
    pub physical_scale_widget: PhysicalScaleWidgetSavedData<modelrdf::TimeUnit>,
}

impl OutputTimeAxisWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::TimeOutputAxis>) -> Self{
        let id_widget = partial.id;
        let description_widget = partial.description;

        let size_widget = partial.size
            .map(|size| OutputSpacetimeSizeWidgetSavedData::from_partial(archive, size))
            .unwrap_or_default();
        let physical_scale_widget = PhysicalScaleWidgetSavedData{
            raw_scale: partial.scale.to_string(),
            unit_widget: 'unit: {
                let Some(raw_unit) = partial.unit else {
                    break 'unit None;
                };
                match raw_unit.parse::<modelrdf::TimeUnit>() {
                    Ok(unit) => Some(unit),
                    Err(e) => {
                        log::warn!("Could not parse time unit '{raw_unit}': {e}");
                        Some(modelrdf::TimeUnit::Second)
                    }
                }
            }
        };
        Self{id_widget, description_widget, size_widget, physical_scale_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct OutputSpaceAxisWidgetSavedData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: OutputSpacetimeSizeWidgetSavedData,
    pub physical_scale_widget: PhysicalScaleWidgetSavedData<modelrdf::SpaceUnit>
}

impl OutputSpaceAxisWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::SpaceOutputAxis>) -> Self{
        Self{
            id_widget: partial.id.to_string(),
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|size| OutputSpacetimeSizeWidgetSavedData::from_partial(archive, size))
                .unwrap_or_default(),
            physical_scale_widget: PhysicalScaleWidgetSavedData {
                raw_scale: partial.scale.to_string(),
                unit_widget: 'unit: {
                    let Some(unit) = partial.unit else {
                        break 'unit None;
                    };
                    let parsed_unit = match unit.parse::<modelrdf::SpaceUnit>() {
                        Ok(parsed_unit) => parsed_unit,
                        Err(e) => {
                            log::warn!("Could not parse spacial unit '{unit}': {e}");
                            break 'unit None
                        },
                    };
                    Some(parsed_unit)
                }
            }
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputAxisWidgetSavedData {
    pub axis_type_widget: AxisType,

    pub batch_axis_widget: BatchAxisWidgetSavedData,
    pub channel_axis_widget: ChannelAxisWidgetSavedData,
    pub index_axis_widget: IndexAxisWidgetSavedData,
    pub space_axis_widget: OutputSpaceAxisWidgetSavedData,
    pub time_axis_widget: OutputTimeAxisWidgetSavedData,
}

impl OutputAxisWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::OutputAxis>) -> Self {
        let mut axis_type_widget = modelrdf::axes::AxisType::Space;
        let batch_axis_widget = match partial.batch {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Batch;
                BatchAxisWidgetSavedData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let channel_axis_widget = match partial.channel{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Channel;
                ChannelAxisWidgetSavedData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let index_axis_widget = match partial.index {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Index;
                IndexAxisWidgetSavedData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let space_axis_widget = match partial.space{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Space;
                OutputSpaceAxisWidgetSavedData::from_partial(archive, partial)
            }
           None => Default::default(),
        };
        let time_axis_widget = match partial.time {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Time;
                OutputTimeAxisWidgetSavedData::from_partial(archive, partial)
            }
           None => Default::default(),
        };
        Self{
            axis_type_widget,
            batch_axis_widget,
            channel_axis_widget,
            index_axis_widget,
            space_axis_widget,
            time_axis_widget,
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum PostprocessingWidgetModeSavedData {
    #[default]
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

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ScaleMeanVarianceWidgetSavedData{
    pub reference_tensor_widget: String,
    pub axes_widget: Option<Vec<String>>,
    pub eps_widget: String,
}

impl ScaleMeanVarianceWidgetSavedData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::postprocessing::ScaleMeanVarianceDescr>) -> Self{
        let reference_tensor_widget = partial.reference_tensor.map(|r| r.to_string()).unwrap_or_default();
        let axes_widget = partial.axes.unwrap_or_default();
        let eps_widget = partial.eps.to_string();
        Self{reference_tensor_widget, axes_widget, eps_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostprocessingWidgetSavedData{
    pub mode: PostprocessingWidgetModeSavedData,
    pub binarize_widget: BinarizePreprocessingWidgetSavedData,
    pub clip_widget: ClipWidgetSavedData,
    pub scale_linear_widget: ScaleLinearWidgetSavedData,
    // pub sigmoid sigmoid has no widget since it has no params
    pub zero_mean_unit_variance_widget: ZeroMeanUnitVarianceWidgetSavedData,
    pub scale_range_widget: ScaleRangeWidgetSavedData,
    pub ensure_dtype_widget: modelrdf::DataType,
    pub fixed_zmuv_widget: FixedZmuvWidgetSavedData,
    pub scale_mean_var_widget: ScaleMeanVarianceWidgetSavedData,
}

impl PostprocessingWidgetSavedData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::postprocessing::PostprocessingDescr>) -> Self{
        let mut mode = PostprocessingWidgetModeSavedData::default();
        let binarize_widget = partial.binarize
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::Binarize;
                BinarizePreprocessingWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let clip_widget = partial.clip
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::Clip;
                ClipWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_linear_widget = partial.scale_linear
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::ScaleLinear;
                ScaleLinearWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let zero_mean_unit_variance_widget = partial.zero_mean_unit_variance
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::ZeroMeanUnitVariance;
                ZeroMeanUnitVarianceWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_range_widget = partial.scale_range
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::ScaleRange;
                ScaleRangeWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let ensure_dtype_widget = partial.ensure_dtype
            .map(|partial_preproc| { 'dtype: {
                mode = PostprocessingWidgetModeSavedData::EnsureDtype;
                let Some(raw_type) = partial_preproc.dtype else{
                    break 'dtype modelrdf::DataType::Float32
                };
                match raw_type.parse::<modelrdf::DataType>() {
                    Ok(dtype) => dtype,
                    Err(e) => {
                        log::warn!("Could not parse dtype '{}': {e}", raw_type);
                        modelrdf::DataType::Float32
                    }
                }
            } })
            .unwrap_or(modelrdf::DataType::Float32);
        let fixed_zmuv_widget = partial.fixed_zero_mean_unit_variance
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::FixedZmuv;
                FixedZmuvWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_mean_var_widget = partial.scale_mean_variance_descr
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeSavedData::ScaleMeanVariance;
                ScaleMeanVarianceWidgetSavedData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        Self{
            mode,
            binarize_widget,
            clip_widget,
            scale_linear_widget,
            zero_mean_unit_variance_widget,
            scale_range_widget,
            ensure_dtype_widget,
            fixed_zmuv_widget,
            scale_mean_var_widget,
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub struct OutputTensorWidgetSavedData {
    pub id_widget: String,
    pub description_widget: String,
    pub axis_widgets: Vec<OutputAxisWidgetSavedData>,
    pub test_tensor_widget: TestTensorWidgetSavedData,
    pub postprocessing_widgets: Vec<CollapsibleWidgetSavedData<PostprocessingWidget>>,
}

impl OutputTensorWidgetSavedData {
    pub fn from_partial<W: std::fmt::Write>(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::OutputTensorDescr>,
        warnings: &mut W,
    ) -> Self {
        let mut id_widget = String::new();
        let mut description_widget = String::new();
        let mut axis_widgets = Vec::<OutputAxisWidgetSavedData>::new();
        let mut postprocessing_widgets = Vec::<CollapsibleWidgetSavedData<PostprocessingWidget>>::new();
        
        if let Some(meta) = partial.metadata {
            if let Some(id) = meta.id {
                id_widget = id;
            }
            if let Some(description) = meta.description {
                description_widget = description;
            }
            if let Some(axes) = meta.axes {
                for partial_axis in axes{
                    axis_widgets.push(OutputAxisWidgetSavedData::from_partial(archive, partial_axis));
                }
            }
            if let Some(preprocs) = meta.postprocessing {
                for partial_preproc in preprocs {
                    let widget = CollapsibleWidgetSavedData::new(
                        PostprocessingWidgetSavedData::from_partial(archive, partial_preproc)
                    );
                    postprocessing_widgets.push(widget);
                }
            }
        }
        let test_tensor_widget = partial.test_tensor
            .map(|tt| TestTensorWidgetSavedData::from_partial(archive, tt, warnings))
            .unwrap_or_default();

        Self{id_widget, description_widget, axis_widgets, test_tensor_widget, postprocessing_widgets}
    }
}

impl OutputTensorWidgetSavedData {
    // pub fn from_partial(archive: &SharedZipArchive, partial: Partial<>) -> Self {
    // }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ModelInterfaceWidgetSavedData {
    pub input_widgets: Vec<InputTensorWidgetSavedData>,
    pub output_widgets: Vec<OutputTensorWidgetSavedData>,
}

impl ModelInterfaceWidgetSavedData {
    pub fn from_partial<W: std::fmt::Write>(
        archive: &SharedZipArchive,
        inputs: Vec<Partial<modelrdf::InputTensorDescr>>,
        outputs: Vec<Partial<modelrdf::OutputTensorDescr>>,
        warnings: &mut W,
    ) -> Self {
        Self{
            input_widgets: inputs.into_iter()
                .map(|i| InputTensorWidgetSavedData::from_partial(archive, i, warnings))
                .collect(),
            output_widgets: outputs.into_iter()
                .map(|o| OutputTensorWidgetSavedData::from_partial(archive, o, warnings))
                .collect(),
        }
    }
}

/// The data that will be persisted to disk when saving the model draft.
/// It is an enum so that newer, incompatible versions can be added as additional
/// variants, and older versions can still be recognized and converted to the newer
/// ones.
#[derive(serde::Serialize, serde::Deserialize, strum::VariantNames)]
#[serde(tag = "app_state_raw_data_version")]
pub enum AppStateSavedData{
    Version1(AppState1SavedData),
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

impl AppStateSavedData{
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
        // FIXME: this app_state_raw_data_version must be manually kep in sync
        // with #[serde(tag=...)]. Maybe a catch-all variant would work?
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
pub struct AppState1SavedData{
    pub staging_name: String,
    pub staging_description: String,
    pub cover_images: Vec<SpecialImageWidgetSavedData>,
    #[serde(default)] // added after AppState1SavedData
    pub model_id_widget: Option<String>,
    pub staging_authors: Vec<AuthorWidgetSavedData>,
    pub attachments_widget: Vec<FileSourceWidgetSavedData>,
    pub staging_citations: Vec<CiteEntryWidgetSavedData>,
    #[serde(default)] // added after AppState1SavedData
    pub custom_config_widget: Option<JsonObjectEditorWidgetSavedData>,
    pub staging_git_repo: Option<String>,
    pub icon_widget: Option<IconWidgetSavedData>,
    #[serde(default)] // added after AppState1SavedData
    pub links_widget: Vec<String>,
    pub staging_maintainers: Vec<MaintainerWidgetSavedData>,
    pub staging_tags: Vec<String>,
    pub staging_version: Option<VersionWidgetSavedData>,
    #[serde(default)]
    pub staging_version_comment: Option<String>,
    pub staging_documentation: CodeEditorWidgetSavedData,
    pub staging_license: ::bioimg_spec::rdf::LicenseId,
    //badges
    pub model_interface_widget: ModelInterfaceWidgetSavedData,
    ////
    pub weights_widget: WeightsWidgetSavedData,
}

impl AppState1SavedData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::ModelRdfV0_5>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        Self{
            staging_name: partial.name.unwrap_or_default(),
            staging_description: partial.description.unwrap_or_default(),
            cover_images: partial.covers.into_iter()
                .map(|ci| SpecialImageWidgetSavedData::from_partial(archive, ci, warnings))
                .collect(),
            model_id_widget: partial.id,
            staging_authors: partial.authors
                .unwrap_or_default()
                .into_iter()
                .map(|partial| AuthorWidgetSavedData::from_partial(archive, partial))
                .collect(),
            attachments_widget: partial.attachments
                .into_iter()
                .map(|partial_fd| FileSourceWidgetSavedData::from_partial_file_descr(archive, partial_fd, warnings))
                .collect(),
            staging_citations: partial.cite
                .unwrap_or_default()
                .into_iter()
                .map(|partial| CiteEntryWidgetSavedData::from_partial(archive, partial))
                .collect(),
            custom_config_widget: Some(JsonObjectEditorWidgetSavedData::from_partial(archive, partial.config)),
            staging_git_repo: partial.git_repo,
            icon_widget: partial.icon.map(|partial| IconWidgetSavedData::from_partial(archive, partial, warnings)),
            links_widget: partial.links,
            staging_maintainers: partial.maintainers.into_iter()
                .map(|partial| MaintainerWidgetSavedData::from_partial(archive, partial))
                .collect(),
            staging_tags: partial.tags,
            staging_version: partial.version.map(|v| VersionWidgetSavedData::from_partial(archive, v)),
            staging_version_comment: partial.version_comment,
            staging_documentation: 'documentation: {
                let Some(doc_file_descr) = partial.documentation else {
                    break 'documentation Default::default();
                };
                let path_in_archive = match rdf::FileReference::try_from(doc_file_descr.clone()){
                    Err(e) => {
                        log::warn!("Can't parse documentation value ({doc_file_descr}) as a file descriptor: {e}");
                        break 'documentation Default::default();
                    },
                    Ok(rdf::FileReference::Url(_url)) => {
                        log::warn!("Can't read documentation from URLs yet"); //FIXME
                        break 'documentation Default::default()
                    },
                    Ok(rdf::FileReference::Path(fspath)) => fspath,
                };
                let doc_bytes = match archive.read_full_entry(&path_in_archive.to_string()){
                    Ok(doc_bytes) => doc_bytes,
                    Err(e) => {
                        log::warn!("Could not read documentation at {}: {e}", path_in_archive);
                        break 'documentation Default::default();
                    }
                };
                let doc_text = match String::from_utf8(doc_bytes) {
                    Ok(doc) => doc,
                    Err(_) => {
                        log::warn!("Could not decode model documentation");
                        break 'documentation Default::default()
                    }
                };
                CodeEditorWidgetSavedData { raw: doc_text }
            },
            staging_license: partial.license
                .map(|raw_license|{
                    match raw_license.parse::<rdf::LicenseId>() {
                        Ok(li) => li,
                        Err(e) => {
                            log::warn!("Could not parse license '{raw_license}': {e}");
                            rdf::LicenseId::MIT
                        }
                    }
                })
                .unwrap_or(rdf::LicenseId::MIT),
            model_interface_widget: ModelInterfaceWidgetSavedData::from_partial(
                archive, partial.inputs.unwrap_or_default(), partial.outputs.unwrap_or_default(), warnings
            ),
            // //badges
            weights_widget: partial.weights
                .map(|w| WeightsWidgetSavedData::from_partial(archive, w, warnings))
                .unwrap_or_default(),
        }
    }
}
