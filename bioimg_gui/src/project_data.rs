use std::path::PathBuf;
use std::sync::Arc;

use ::aspartial::AsPartial;
use serde_json::Value as JsVal;

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

/// Produces a json value from `a - b`, that is,
/// "what parts of `a` are contemplated in `b`". If `b` has data
/// without a correspondent in `a`, those will just be ignored
pub fn json_diff(a: JsVal, b: JsVal) -> Option<JsVal> {
    match (a, b) {
        (JsVal::Object(mut a_obj), JsVal::Object(b_obj)) => {
            for (b_member_key, b_member_val) in b_obj.into_iter(){
                let Some(a_member_val) = a_obj.remove(&b_member_key) else {
                    continue
                };
                let Some(member_diff) = json_diff(a_member_val, b_member_val) else {
                    continue
                };
                a_obj.insert(b_member_key, member_diff);
            }
            if a_obj.len() > 0 {
                Some(JsVal::Object(a_obj))
            } else {
                None
            }
        },
        (JsVal::Array(a_arr), JsVal::Array(b_arr)) => {
            let mut diffs = Vec::<JsVal>::with_capacity(a_arr.len());
            let mut a_iter = a_arr.into_iter();
            let mut b_iter = b_arr.into_iter();
            while let Some(a_item) = a_iter.next(){
                let Some(b_item) = b_iter.next() else {
                    diffs.push(a_item);
                    continue
                };
                if let Some(diff) = json_diff(a_item, b_item) {
                    diffs.push(diff)
                }
            }
            if diffs.is_empty() {
                None
            } else {
                Some(JsVal::Array(diffs))
            }
        },
        (JsVal::String(a_leaf), JsVal::String(b_leaf)) => {
            if a_leaf == b_leaf {
                None
            } else {
                Some(JsVal::String(a_leaf))
            }
        },
        (JsVal::Number(a_leaf), JsVal::Number(b_leaf)) => {
            if a_leaf == b_leaf {
                None
            } else {
                Some(JsVal::Number(a_leaf))
            }
        },
        (JsVal::Null, JsVal::Null) => {
            None
        },
        (a, _) => {
            Some(a)
        }
    }
}

#[test]
fn test_json_diff(){
    let x = serde_json::json!(
        {
            "a": 123,
            "b": {
                "inner_a": [1,2,3],
                "inner_b": 3.14,
            }
        }
    );
    let y = serde_json::json!(
        {
            "b": {
                "inner_a": [7,2],
            }
        }
    );
    let expected_diff = serde_json::json!(
        {
            "a": 123,
            "b": {
                "inner_a": [1,3],
                "inner_b": 3.14,
            }
        }
    );

    let diff = json_diff(x, y).unwrap();
    println!("diff:\n{}", serde_json::to_string_pretty(&diff).unwrap());
    assert_eq!(expected_diff, diff);
}


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

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum FileWidgetRawData{
    #[default]
    Empty,
    AboutToLoad{path: PathBuf},
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum TestTensorWidgetRawData{
    #[default]
    Empty,
    Loaded{path: Option<PathBuf>, data: Vec<u8>},
}

impl TestTensorWidgetRawData {
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
pub enum LocalFileSourceWidgetRawData{
    #[default]
    Empty,
    InMemoryData{name: Option<String>, data: Arc<[u8]>},
    AboutToLoad{path: String, inner_path: Option<String>}
}

impl LocalFileSourceWidgetRawData{
    pub fn from_partial<W: std::fmt::Write>(archive: &SharedZipArchive, raw_path: String, warnings: &mut W) -> Self{
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
                _ = writeln!(warnings, "Could not load contents of {raw_path}/{zip_entry_path}: {e}");
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
    fn from_partial<W: std::fmt::Write>(archive: &SharedZipArchive, partial: String, warnings: &mut W) -> Self{
        if let Ok(url) = rdf::HttpUrl::try_from(partial.clone()) { //FIXME: parse?
            return Self::Url(url.to_string())
        };
        Self::Local(LocalFileSourceWidgetRawData::from_partial(archive, partial, warnings))
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
pub enum ImageWidget2LoadingStateRawData{
    Empty,
    Forced{img_bytes: Vec<u8>}
}

// impl ImageWidget2LoadingStateRawData {
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
pub struct ImageWidget2RawData{
    pub file_source_widget: FileSourceWidgetRawData,
    pub loading_state: ImageWidget2LoadingStateRawData,
}

impl ImageWidget2RawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: String, warnings: &mut impl std::fmt::Write) -> Self {
        let file_source_state = FileSourceWidgetRawData::from_partial(archive, partial.clone(), warnings);
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
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: String,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        Self { image_widget: ImageWidget2RawData::from_partial(archive, partial, warnings) }
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum IconWidgetRawData{
    Emoji(String),
    Image(SpecialImageWidgetRawData),
}

impl IconWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::Icon as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        if archive.has_entry(&partial) {
            let img_data = SpecialImageWidgetRawData::from_partial(archive, partial, warnings);
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

impl<Inner: Restore> CollapsibleWidgetRawData<Inner> {
    pub fn new(inner: Inner::RawData) -> Self {
        Self{inner, is_closed: true}
    }
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

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct PhysicalScaleWidgetRawData<T>{
    pub raw_scale: String,
    pub unit_widget: Option<T>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct BatchAxisWidgetRawData{
    pub description_widget: String,
    pub staging_allow_auto_size: bool,
}

impl BatchAxisWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::BatchAxis>) -> Self{
        Self{
            description_widget: partial.description.to_string(),
            staging_allow_auto_size: partial.size.is_some(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum ChannelNamesModeRawData{
    #[default]
    Explicit,
    Pattern,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum AxisSizeModeRawData{
    #[default]
    Fixed,
    Reference,
    Parameterized,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ParameterizedAxisSizeWidgetRawData {
    pub staging_min: usize,
    pub staging_step: usize,
}

impl ParameterizedAxisSizeWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::ParameterizedAxisSize>) -> Self {
        Self{
            staging_min: partial.min.map(|min| usize::from(min)).unwrap_or(0),
            staging_step: partial.step.map(|step| usize::from(step)).unwrap_or(0),
        }
    } 
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AnyAxisSizeWidgetRawData {
    pub mode: AxisSizeModeRawData,

    pub staging_fixed_size: usize,
    pub staging_size_ref: AxisSizeReferenceWidgetRawData,
    pub staging_parameterized: ParameterizedAxisSizeWidgetRawData,
}

impl AnyAxisSizeWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::AnyAxisSize>) -> Self {
        let mut mode = AxisSizeModeRawData::Fixed;
        let mut staging_fixed_size = 0usize;
        let mut staging_size_ref = AxisSizeReferenceWidgetRawData::default();
        let mut staging_parameterized = ParameterizedAxisSizeWidgetRawData::default();
        
        if let Some(fixed) = partial.fixed {
            mode = AxisSizeModeRawData::Fixed;
            staging_fixed_size = fixed.into();
        }
        if let Some(refer) = partial.reference {
            if refer.qualified_axis_id.is_some(){
                mode = AxisSizeModeRawData::Reference;
            }
            staging_size_ref = AxisSizeReferenceWidgetRawData::from_partial(archive, refer);
        }
        if let Some(params) = partial.parameterized {
            if params.step.is_some() || params.step.is_some(){
                mode = AxisSizeModeRawData::Parameterized;
            }
            staging_parameterized = ParameterizedAxisSizeWidgetRawData::from_partial(archive, params);
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
pub struct IndexAxisWidgetRawData {
    pub description_widget: String,
    pub size_widget: AnyAxisSizeWidgetRawData,
}

impl IndexAxisWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::IndexAxis>) -> Self {
        Self{
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|partial| AnyAxisSizeWidgetRawData::from_partial(archive, partial))
                .unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AxisSizeReferenceWidgetRawData {
    pub staging_tensor_id: String,
    pub staging_axis_id: String,
    pub staging_offset: usize,
}

impl AxisSizeReferenceWidgetRawData {
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
pub struct ChannelAxisWidgetRawData {
    pub description_widget: String,

    pub channel_names_mode_widget: ChannelNamesModeRawData,
    pub channel_extent_widget: usize,
    pub channel_name_prefix_widget: String,
    pub channel_name_suffix_widget: String,

    pub staging_explicit_names: Vec<String>,
}

impl ChannelAxisWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<rdf::model::ChannelAxis>) -> Self {
        Self{
            description_widget: partial.description.to_string(),
            channel_extent_widget: partial.channel_names.as_ref().map(|names| names.len()).unwrap_or_default(),
            channel_names_mode_widget: ChannelNamesModeRawData::Explicit,
            staging_explicit_names: match partial.channel_names {
                Some(names) => names.into_iter().map(|name| name.to_string()).collect(),
                None => vec![]
            },
            ..Default::default()
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct InputSpaceAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: AnyAxisSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::SpaceUnit>
}

impl InputSpaceAxisWidgetRawData {
    pub fn from_partial<W: std::fmt::Write>(archive: &SharedZipArchive, partial: Partial<rdf::model::SpaceInputAxis>, warnings: &mut W) -> Self{
        Self{
            id_widget: partial.id.to_string(),
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|size| AnyAxisSizeWidgetRawData::from_partial(archive, size))
                .unwrap_or_default(),
            physical_scale_widget: PhysicalScaleWidgetRawData {
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
pub struct InputTimeAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: AnyAxisSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::TimeUnit>,
}

impl InputTimeAxisWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::TimeInputAxis>) -> Self{
        Self{
            id_widget: partial.id.to_string(),
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|size| AnyAxisSizeWidgetRawData::from_partial(archive, size))
                .unwrap_or_default(),
            physical_scale_widget: PhysicalScaleWidgetRawData {
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
pub struct InputAxisWidgetRawData {
    pub axis_type_widget: bioimg_spec::rdf::model::axes::AxisType,
    pub batch_axis_widget: BatchAxisWidgetRawData,
    pub channel_axis_widget: ChannelAxisWidgetRawData,
    pub index_axis_widget: IndexAxisWidgetRawData,
    pub space_axis_widget: InputSpaceAxisWidgetRawData,
    pub time_axis_widget: InputTimeAxisWidgetRawData,
}

impl InputAxisWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::InputAxis>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let mut axis_type_widget = modelrdf::axes::AxisType::Space;
        let batch_axis_widget = match partial.batch {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Batch;
                BatchAxisWidgetRawData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let channel_axis_widget = match partial.channel{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Channel;
                ChannelAxisWidgetRawData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let index_axis_widget = match partial.index {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Index;
                IndexAxisWidgetRawData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let space_axis_widget = match partial.space{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Space;
                InputSpaceAxisWidgetRawData::from_partial(archive, partial, warnings)
            }
           None => Default::default(),
        };
        let time_axis_widget = match partial.time {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Time;
                InputTimeAxisWidgetRawData::from_partial(archive, partial)
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
pub struct WeightsDescrBaseWidgetRawData{
    pub source_widget: FileSourceWidgetRawData,
    pub authors_widget: Option<Vec<CollapsibleWidgetRawData<AuthorWidget>>>,
}

impl WeightsDescrBaseWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: <rdf::model::WeightsDescrBase as AsPartial>::Partial,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let source = partial.source
            .map(|src| FileSourceWidgetRawData::from_partial(archive, src, warnings))
            .unwrap_or_default();
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
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<rdf::model::TorchscriptWeightsDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let base = partial.base.map(|partial| WeightsDescrBaseWidgetRawData::from_partial(archive, partial, warnings)).unwrap_or_default();
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
        warnings: &mut impl std::fmt::Write,
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
            source_widget = from_file_descr.file_descr
                .map(|fd| {
                    let Some(src) = fd.source else {
                        return Default::default();
                    };
                    FileSourceWidgetRawData::from_partial(archive, src, warnings)
                })
                .unwrap_or_default();
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
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetRawData::from_partial(archive, base, warnings))
            .unwrap_or_default();
        let architecture = partial.architecture
            .map(|arch| PytorchArchWidgetRawData::from_partial(archive, arch, warnings))
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
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetRawData::from_partial(archive, base, warnings))
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
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let base = partial.base
            .map(|base| WeightsDescrBaseWidgetRawData::from_partial(archive, base, warnings))
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
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<rdf::model::WeightsDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        let keras = partial.keras_hdf5.map(|partial| {
            let weights = KerasHdf5WeightsWidgetRawData::from_partial(archive, partial, warnings);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        let torchscript = partial.torchscript.map(|partial| {
            let weights = TorchscriptWeightsWidgetRawData::from_partial(archive, partial, warnings);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        let pytorch_state_dict = partial.pytorch_state_dict.map(|partial|{
            let weights = PytorchStateDictWidgetRawData::from_partial(archive, partial, warnings);
            CollapsibleWidgetRawData{is_closed: true, inner: weights}
        });
        let onnx = partial.onnx.map(|partial|{
            let weights = OnnxWeightsWidgetRawData::from_partial(archive, partial, warnings);
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

impl InputTensorWidgetRawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::InputTensorDescr>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self {
        let mut id_widget = String::new();
        let mut is_optional = false;
        let mut description_widget = String::new();
        let mut axis_widgets = Vec::<InputAxisWidgetRawData>::new();
        let mut preprocessing_widget = Vec::<PreprocessingWidgetRawData>::new();
        
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
                    axis_widgets.push(InputAxisWidgetRawData::from_partial(archive, partial_axis, warnings));
                }
            }
            if let Some(preprocs) = meta.preprocessing {
                for partial_preproc in preprocs {
                    preprocessing_widget.push(PreprocessingWidgetRawData::from_partial(archive, partial_preproc));
                }
            }
        }
        let test_tensor_widget = partial.test_tensor
            .map(|tt| TestTensorWidgetRawData::from_partial(archive, tt, warnings))
            .unwrap_or_default();

        Self{id_widget, is_optional, description_widget, axis_widgets, test_tensor_widget, preprocessing_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum PreprocessingWidgetModeRawData {
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
pub enum BinarizeModeRawData{
    #[default]
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SimpleBinarizeWidgetRawData{
    pub threshold_widget: String,
}

impl SimpleBinarizeWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::SimpleBinarizeDescr>) -> Self {
        Self{ threshold_widget: partial.threshold.map(|t| t.to_string()).unwrap_or_default() }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct BinarizeAlongAxisWidgetRawData{
    pub thresholds_widget: Vec<String>,
    pub axis_id_widget: String,
}

impl BinarizeAlongAxisWidgetRawData {
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
pub struct BinarizePreprocessingWidgetRawData{
    pub mode: BinarizeModeRawData,
    pub simple_binarize_widget: SimpleBinarizeWidgetRawData,
    pub binarize_along_axis_wiget: BinarizeAlongAxisWidgetRawData,
}

impl BinarizePreprocessingWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::BinarizeDescr>) -> Self {
        let mut mode = BinarizeModeRawData::Simple;
        let simple_binarize_widget = match partial.simple {
            Some(simple) => {
                mode = BinarizeModeRawData::Simple;
                SimpleBinarizeWidgetRawData::from_partial(archive, simple)
            },
            None => SimpleBinarizeWidgetRawData::default(),
        };
        let binarize_along_axis_wiget = match partial.along_axis {
            Some(along_axis) => {
                mode = BinarizeModeRawData::AlongAxis;
                BinarizeAlongAxisWidgetRawData::from_partial(archive, along_axis)
            },
            None => BinarizeAlongAxisWidgetRawData::default(),
        };
        Self{mode, simple_binarize_widget, binarize_along_axis_wiget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ClipWidgetRawData{
    pub min_widget: String,
    pub max_widget: String,
}

impl ClipWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ClipDescr>) -> Self {
        Self{
            min_widget: partial.min.map(|min| min.to_string()).unwrap_or_default(),
            max_widget: partial.max.map(|max| max.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub enum ScaleLinearModeRawData{
    #[default]
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SimpleScaleLinearWidgetRawData{
    pub gain_widget: String,
    pub offset_widget: String,
}

impl SimpleScaleLinearWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::SimpleScaleLinearDescr>) -> Self {
        Self{
            gain_widget: partial.gain.to_string(),
            offset_widget: partial.offset.to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct ScaleLinearAlongAxisWidgetRawData{
    pub axis_widget: String,
    pub gain_offsets_widget: Vec<SimpleScaleLinearWidgetRawData>,
}

impl ScaleLinearAlongAxisWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleLinearAlongAxisDescr>) -> Self {
        let mut gains = match partial.gain {
            PartialSingleOrMultiple::Single(item) => vec![item],
            PartialSingleOrMultiple::Multiple(items) => items,
        }.into_iter();
        let mut offsets = match partial.offset {
            PartialSingleOrMultiple::Single(item) => vec![item],
            PartialSingleOrMultiple::Multiple(items) => items,
        }.into_iter();

        let mut gain_offsets_widget = Vec::<SimpleScaleLinearWidgetRawData>::new();
        loop {
            let gain = gains.next();
            let offset = offsets.next();
            if gain.is_none() && offset.is_none() {
                break
            }
            gain_offsets_widget.push(
                SimpleScaleLinearWidgetRawData {
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
pub struct ZeroMeanUnitVarianceWidgetRawData{
    pub axes_widget: Option<Vec<String>>,
    pub epsilon_widget: String,
}

impl ZeroMeanUnitVarianceWidgetRawData {
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
pub struct PercentilesWidgetRawData{
    pub min_widget: String,
    pub max_widget: String,
}

impl PercentilesWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleRangePercentile>) -> Self {
        Self{
            min_widget: partial.min_percentile.map(|v| v.to_string()).unwrap_or_default(),
            max_widget: partial.max_percentile.map(|v| v.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ScaleRangeWidgetRawData{
    pub axes_widget: Option<Vec<String>>,
    pub percentiles_widget: PercentilesWidgetRawData,
    pub epsilon_widget: String,
    pub reference_tensor: Option<String>,
}

impl ScaleRangeWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleRangeDescr>) -> Self {
        let axes = match partial.axes {
            Some(Some(axes)) => match axes.len() {
                0 => None,
                _ => Some(axes),
            },
            _ => None,
        };
        let percentiles_widget = partial.percentiles
            .map(|per| PercentilesWidgetRawData::from_partial(archive, per))
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
pub enum ZmuvWidgetModeRawData{
    #[default]
    Simple,
    AlongAxis,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct SimpleFixedZmuvWidgetRawData{
    pub mean_widget: String,
    pub std_widget: String,
}

impl SimpleFixedZmuvWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::SimpleFixedZmuv>) -> Self {
        Self{
            mean_widget: partial.mean.map(|v| v.to_string()).unwrap_or_default(),
            std_widget: partial.std.map(|v| v.to_string()).unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct FixedZmuvAlongAxisWidgetRawData{
    pub axis_widget: String,
    pub mean_and_std_widget: Vec<SimpleFixedZmuvWidgetRawData>,
}

impl FixedZmuvAlongAxisWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::FixedZmuvAlongAxis>) -> Self {
        let mut means = partial.mean.unwrap_or_default().into_iter();
        let mut stds = partial.std.unwrap_or_default().into_iter();

        let mut mean_and_std_widget = Vec::<SimpleFixedZmuvWidgetRawData >::new();
        loop {
            let mean = means.next();
            let std = stds.next();
            if mean.is_none() && std.is_none() {
                break
            }
            mean_and_std_widget.push(
                SimpleFixedZmuvWidgetRawData {
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
pub struct FixedZmuvWidgetRawData{
    pub mode_widget: ZmuvWidgetModeRawData,
    pub simple_widget: SimpleFixedZmuvWidgetRawData,
    pub along_axis_widget: FixedZmuvAlongAxisWidgetRawData,
}

impl FixedZmuvWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::FixedZmuv>) -> Self {
        let mut mode_widget = ZmuvWidgetModeRawData::Simple;
        let simple_widget = partial.simple
            .map(|partial_preproc| {
                mode_widget = ZmuvWidgetModeRawData::Simple;
                SimpleFixedZmuvWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let along_axis_widget = partial.along_axis
            .map(|partial_preproc| {
                mode_widget = ZmuvWidgetModeRawData::AlongAxis;
                FixedZmuvAlongAxisWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        Self{mode_widget, simple_widget, along_axis_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ScaleLinearWidgetRawData{
    pub mode: ScaleLinearModeRawData,
    pub simple_widget: SimpleScaleLinearWidgetRawData,
    pub along_axis_widget: ScaleLinearAlongAxisWidgetRawData,
}

impl ScaleLinearWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::preprocessing::ScaleLinearDescr>) -> Self {
        let mut mode = ScaleLinearModeRawData::Simple;
        let simple_widget = partial.simple
            .map(|partial_preproc| {
                mode = ScaleLinearModeRawData::Simple;
                SimpleScaleLinearWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let along_axis_widget = partial.along_axis
            .map(|partial_preproc| {
                mode = ScaleLinearModeRawData::AlongAxis;
                ScaleLinearAlongAxisWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        Self{mode, simple_widget, along_axis_widget}
    }
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

impl PreprocessingWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::PreprocessingDescr>) -> Self{
        let mut mode = PreprocessingWidgetModeRawData::default();
        let binarize_widget = partial.binarize
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeRawData::Binarize;
                BinarizePreprocessingWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let clip_widget = partial.clip
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeRawData::Clip;
                ClipWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_linear_widget = partial.scale_linear
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeRawData::ScaleLinear;
                ScaleLinearWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let zero_mean_unit_variance_widget = partial.zero_mean_unit_variance
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeRawData::ZeroMeanUnitVariance;
                ZeroMeanUnitVarianceWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_range_widget = partial.scale_range
            .map(|partial_preproc| {
                mode = PreprocessingWidgetModeRawData::ScaleRange;
                ScaleRangeWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let ensure_dtype_widget = partial.ensure_dtype
            .map(|partial_preproc| { 'dtype: {
                mode = PreprocessingWidgetModeRawData::EnsureDtype;
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
                mode = PreprocessingWidgetModeRawData::FixedZmuv;
                FixedZmuvWidgetRawData::from_partial(archive, partial_preproc)
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
pub struct OutputSpacetimeSizeWidgetRawData{
    pub has_halo: bool,
    pub halo_widget: u64,
    pub size_widget: AnyAxisSizeWidgetRawData,
}

impl OutputSpacetimeSizeWidgetRawData {
    fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::axes::output_axes::OutputSpacetimeSize>) -> Self {
        let mut has_halo = false;
        let mut halo_widget = 0u64;
        let mut size_widget = AnyAxisSizeWidgetRawData::default();

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
                    size_widget.staging_size_ref = AxisSizeReferenceWidgetRawData::from_partial(archive, refer);
                }
            }
        }
        if let Some(standard) = partial.standard {
            if let Some(size) = standard.size{
                //FIXME: detect setting size multiple times?
                size_widget = AnyAxisSizeWidgetRawData::from_partial(archive, size);
            }
        }

        Self{has_halo, halo_widget, size_widget}
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct OutputTimeAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: OutputSpacetimeSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::TimeUnit>,
}

impl OutputTimeAxisWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::TimeOutputAxis>) -> Self{
        let id_widget = partial.id;
        let description_widget = partial.description;

        let size_widget = partial.size
            .map(|size| OutputSpacetimeSizeWidgetRawData::from_partial(archive, size))
            .unwrap_or_default();
        let physical_scale_widget = PhysicalScaleWidgetRawData{
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
pub struct OutputSpaceAxisWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,

    pub size_widget: OutputSpacetimeSizeWidgetRawData,
    pub physical_scale_widget: PhysicalScaleWidgetRawData<modelrdf::SpaceUnit>
}

impl OutputSpaceAxisWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<rdf::model::SpaceOutputAxis>) -> Self{
        Self{
            id_widget: partial.id.to_string(),
            description_widget: partial.description.to_string(),
            size_widget: partial.size
                .map(|size| OutputSpacetimeSizeWidgetRawData::from_partial(archive, size))
                .unwrap_or_default(),
            physical_scale_widget: PhysicalScaleWidgetRawData {
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
pub struct OutputAxisWidgetRawData {
    pub axis_type_widget: AxisType,

    pub batch_axis_widget: BatchAxisWidgetRawData,
    pub channel_axis_widget: ChannelAxisWidgetRawData,
    pub index_axis_widget: IndexAxisWidgetRawData,
    pub space_axis_widget: OutputSpaceAxisWidgetRawData,
    pub time_axis_widget: OutputTimeAxisWidgetRawData,
}

impl OutputAxisWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::OutputAxis>) -> Self {
        let mut axis_type_widget = modelrdf::axes::AxisType::Space;
        let batch_axis_widget = match partial.batch {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Batch;
                BatchAxisWidgetRawData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let channel_axis_widget = match partial.channel{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Channel;
                ChannelAxisWidgetRawData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let index_axis_widget = match partial.index {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Index;
                IndexAxisWidgetRawData::from_partial(archive, partial)
            }
            None => Default::default(),
        };
        let space_axis_widget = match partial.space{
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Space;
                OutputSpaceAxisWidgetRawData::from_partial(archive, partial)
            }
           None => Default::default(),
        };
        let time_axis_widget = match partial.time {
            Some(partial) => {
                axis_type_widget = modelrdf::axes::AxisType::Time;
                OutputTimeAxisWidgetRawData::from_partial(archive, partial)
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
pub enum PostprocessingWidgetModeRawData {
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
pub struct ScaleMeanVarianceWidgetRawData{
    pub reference_tensor_widget: String,
    pub axes_widget: Option<Vec<String>>,
    pub eps_widget: String,
}

impl ScaleMeanVarianceWidgetRawData {
    pub fn from_partial(_archive: &SharedZipArchive, partial: Partial<modelrdf::postprocessing::ScaleMeanVarianceDescr>) -> Self{
        let reference_tensor_widget = partial.reference_tensor.map(|r| r.to_string()).unwrap_or_default();
        let axes_widget = partial.axes.unwrap_or_default();
        let eps_widget = partial.eps.to_string();
        Self{reference_tensor_widget, axes_widget, eps_widget}
    }
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

impl PostprocessingWidgetRawData {
    pub fn from_partial(archive: &SharedZipArchive, partial: Partial<modelrdf::postprocessing::PostprocessingDescr>) -> Self{
        let mut mode = PostprocessingWidgetModeRawData::default();
        let binarize_widget = partial.binarize
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeRawData::Binarize;
                BinarizePreprocessingWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let clip_widget = partial.clip
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeRawData::Clip;
                ClipWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_linear_widget = partial.scale_linear
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeRawData::ScaleLinear;
                ScaleLinearWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let zero_mean_unit_variance_widget = partial.zero_mean_unit_variance
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeRawData::ZeroMeanUnitVariance;
                ZeroMeanUnitVarianceWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_range_widget = partial.scale_range
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeRawData::ScaleRange;
                ScaleRangeWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let ensure_dtype_widget = partial.ensure_dtype
            .map(|partial_preproc| { 'dtype: {
                mode = PostprocessingWidgetModeRawData::EnsureDtype;
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
                mode = PostprocessingWidgetModeRawData::FixedZmuv;
                FixedZmuvWidgetRawData::from_partial(archive, partial_preproc)
            })
            .unwrap_or_default();
        let scale_mean_var_widget = partial.scale_mean_variance_descr
            .map(|partial_preproc| {
                mode = PostprocessingWidgetModeRawData::ScaleMeanVariance;
                ScaleMeanVarianceWidgetRawData::from_partial(archive, partial_preproc)
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
pub struct OutputTensorWidgetRawData {
    pub id_widget: String,
    pub description_widget: String,
    pub axis_widgets: Vec<OutputAxisWidgetRawData>,
    pub test_tensor_widget: TestTensorWidgetRawData,
    pub postprocessing_widgets: Vec<CollapsibleWidgetRawData<PostprocessingWidget>>,
}

impl OutputTensorWidgetRawData {
    pub fn from_partial<W: std::fmt::Write>(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::OutputTensorDescr>,
        warnings: &mut W,
    ) -> Self {
        let mut id_widget = String::new();
        let mut description_widget = String::new();
        let mut axis_widgets = Vec::<OutputAxisWidgetRawData>::new();
        let mut postprocessing_widgets = Vec::<CollapsibleWidgetRawData<PostprocessingWidget>>::new();
        
        if let Some(meta) = partial.metadata {
            if let Some(id) = meta.id {
                id_widget = id;
            }
            if let Some(description) = meta.description {
                description_widget = description;
            }
            if let Some(axes) = meta.axes {
                for partial_axis in axes{
                    axis_widgets.push(OutputAxisWidgetRawData::from_partial(archive, partial_axis));
                }
            }
            if let Some(preprocs) = meta.postprocessing {
                for partial_preproc in preprocs {
                    let widget = CollapsibleWidgetRawData::new(
                        PostprocessingWidgetRawData::from_partial(archive, partial_preproc)
                    );
                    postprocessing_widgets.push(widget);
                }
            }
        }
        let test_tensor_widget = partial.test_tensor
            .map(|tt| TestTensorWidgetRawData::from_partial(archive, tt, warnings))
            .unwrap_or_default();

        Self{id_widget, description_widget, axis_widgets, test_tensor_widget, postprocessing_widgets}
    }
}

impl OutputTensorWidgetRawData {
    // pub fn from_partial(archive: &SharedZipArchive, partial: Partial<>) -> Self {
    // }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ModelInterfaceWidgetRawData {
    pub input_widgets: Vec<InputTensorWidgetRawData>,
    pub output_widgets: Vec<OutputTensorWidgetRawData>,
}

impl ModelInterfaceWidgetRawData {
    pub fn from_partial<W: std::fmt::Write>(
        archive: &SharedZipArchive,
        inputs: Vec<Partial<modelrdf::InputTensorDescr>>,
        outputs: Vec<Partial<modelrdf::OutputTensorDescr>>,
        warnings: &mut W,
    ) -> Self {
        Self{
            input_widgets: inputs.into_iter()
                .map(|i| InputTensorWidgetRawData::from_partial(archive, i, warnings))
                .collect(),
            output_widgets: outputs.into_iter()
                .map(|o| OutputTensorWidgetRawData::from_partial(archive, o, warnings))
                .collect(),
        }
    }
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

impl AppState1RawData {
    pub fn from_partial(
        archive: &SharedZipArchive,
        partial: Partial<modelrdf::ModelRdfV0_5>,
        warnings: &mut impl std::fmt::Write,
    ) -> Self{
        Self{
            staging_name: partial.name.unwrap_or_default(),
            staging_description: partial.description.unwrap_or_default(),
            cover_images: partial.covers.into_iter()
                .map(|ci| SpecialImageWidgetRawData::from_partial(archive, ci, warnings))
                .collect(),
            model_id_widget: partial.id,
            staging_authors: partial.authors
                .unwrap_or_default()
                .into_iter()
                .map(|partial| AuthorWidgetRawData::from_partial(archive, partial))
                .collect(),
            attachments_widget: partial.attachments
                .into_iter()
                .map(|partial_fd| FileSourceWidgetRawData::from_partial_file_descr(archive, partial_fd, warnings))
                .collect(),
            staging_citations: partial.cite
                .unwrap_or_default()
                .into_iter()
                .map(|partial| CiteEntryWidgetRawData::from_partial(archive, partial))
                .collect(),
            custom_config_widget: Some(JsonObjectEditorWidgetRawData::from_partial(archive, partial.config)),
            staging_git_repo: partial.git_repo,
            icon_widget: partial.icon.map(|partial| IconWidgetRawData::from_partial(archive, partial, warnings)),
            links_widget: partial.links,
            staging_maintainers: partial.maintainers.into_iter()
                .map(|partial| MaintainerWidgetRawData::from_partial(archive, partial))
                .collect(),
            staging_tags: partial.tags,
            staging_version: partial.version.map(|v| VersionWidgetRawData::from_partial(archive, v)),
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
                CodeEditorWidgetRawData { raw: doc_text }
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
            model_interface_widget: ModelInterfaceWidgetRawData::from_partial(
                archive, partial.inputs.unwrap_or_default(), partial.outputs.unwrap_or_default(), warnings
            ),
            // //badges
            weights_widget: partial.weights
                .map(|w| WeightsWidgetRawData::from_partial(archive, w, warnings))
                .unwrap_or_default(),
        }
    }
}
