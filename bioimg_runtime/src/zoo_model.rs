use std::{
    io::{Read, Seek, Write}, path::{Path, PathBuf}, sync::Arc
};

use bioimg_spec::rdf::{model::ModelRdfName, FileReference, FsPath, HttpUrl, LicenseId, ResourceId, Version};
use bioimg_spec::rdf::model::unsupported::Version_0_5_4_OrLater;
use bioimg_spec::rdf::model::unsupported::UnsupportedLegacyModel;
use bioimg_spec::rdf::model::unsupported::UnsupportedFutureModel;
use bioimg_spec::rdf;
use bioimg_spec::rdf::version::Version_0_5_x;
use bioimg_spec::rdf::non_empty_list::NonEmptyList;
use bioimg_spec::rdf::model::RdfTypeModel;
use bioimg_spec::rdf::model::unsupported::Version_0_4_X_OrEarlier;
use bioimg_spec::rdf::model::ModelRdfV0_5;
use bioimg_spec::rdf::maintainer::Maintainer;
use bioimg_spec::rdf::file_reference::FsPathComponent;
use bioimg_spec::rdf::author::Author2;
use bioimg_spec::rdf::model as  modelrdf;
use image::ImageError;

use crate::{FileSource, Icon, ModelInterface, NpyArray, TensorValidationError};
use crate::zip_archive_ext::{SharedZipArchive, ZipArchiveOpenError};
use crate::cover_image::CoverImageLoadingError;
use crate::CoverImage;
use crate::zip_writer_ext::ModelZipWriter;
use crate::npy_array::ArcNpyArray;
use crate::model_weights::{ModelWeights, ModelWeightsLoadingError};
use crate::model_interface::{InputSlot, ModelInterfaceLoadingError, OutputSlot};
use crate::icon::IconLoadingError;

#[derive(thiserror::Error, Debug)]
pub enum ModelPackingError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    ImageError(#[from] ImageError),
    #[error("{0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("File {0} already exists")]
    AlreadyExists(PathBuf),
    #[error("{0}")]
    WriteNpyError(#[from] ndarray_npy::WriteNpyError),
    #[error("{0}")]
    RdfSerializationError(#[from] serde_json::Error),
    #[error("Could not write yaml file to zip: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error("Http error: {reason}")]
    HttpErro{reason: String},
    #[error("Unexpected status ({status})when requesting {url}")]
    UnexpectedHttpStatus{status: u16, url: HttpUrl},
}

#[derive(thiserror::Error, Debug)]
pub enum ModelLoadingError{
    #[error("Error reading file: {0}")]
    IoErro(#[from] std::io::Error),
    #[error("rdf.yaml file not found")]
    RdfYamlNotFound,
    #[error("{0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error(transparent)]
    ZipArchiveOpen(#[from] ZipArchiveOpenError),
    #[error("Could not parse model rdf as yaml: {0}")]
    YamlParsingError(#[from] serde_yaml::Error),
    #[error("Could not load a cover image: {0}")]
    CoverImageLoadingError(#[from] CoverImageLoadingError),
    #[error("Could not load an icon: {0}")]
    IconLoadingError(#[from] IconLoadingError),
    #[error("Url file reference not supported yet")]
    UrlFileReferenceNotSupportedYet,
    #[error("Error loading models from rdf: {0}")]
    ModelWeightsLoadingError(#[from] ModelWeightsLoadingError),
    #[error("Could not load model interface: {0}")]
    ModelInterfaceLoadingError(#[from] ModelInterfaceLoadingError),
    #[error("Could not produce a valid Input tensor description: {0}")]
    InputTensorParsingError(#[from] modelrdf::input_tensor::InputTensorParsingError),
    #[error("Invalid input/output configurtation: {0}")]
    TensorValidationError(#[from] TensorValidationError),
    #[error("Unsupported legacy model version: {version}. The earliest supported version is {earliest_supported}")]
    UnsupportedLegacyModel{version: Version_0_4_X_OrEarlier, earliest_supported: Version},
    #[error("Rdf version is too new for this application: {format_version}. The latest supported version is {latest_supported}")]
    FutureModel{format_version: Version_0_5_4_OrLater, latest_supported: Version},
    #[error("Bad rdf: {inner}")]
    BadModel{inner: serde_yaml::Error},
    #[error("Unrecognized rdf data (found version {format_version:?})")]
    UnrecognizedRdf{format_version: Option<String>},
}

pub struct ZooModel {
    pub description: rdf::ResourceTextDescription,
    pub covers: Vec<CoverImage>,
    pub attachments: Vec<FileSource>,
    pub cite: NonEmptyList<rdf::CiteEntry2>,
    pub config: serde_json::Map<String, serde_json::Value>,
    pub git_repo: Option<rdf::HttpUrl>,
    pub icon: Option<Icon>,
    pub links: Vec<String>,
    pub maintainers: Vec<Maintainer>,
    pub tags: Vec<rdf::Tag>,
    pub version: Option<Version>,
    pub version_comment: Option<BoundedString<0, 512>>,
    pub authors: NonEmptyList<Author2>,
    pub documentation: String,
    pub license: LicenseId,
    pub name: ModelRdfName,
    pub id: Option<ResourceId>,
    // training_data: DatasetDescrEnum, //FIXME
    pub weights: ModelWeights,
    pub interface: ModelInterface<ArcNpyArray>,
}

impl ZooModel{
    pub fn try_load(path: &Path) -> Result<Self, ModelLoadingError>{
        let archive = SharedZipArchive::open(path)?;
        Self::try_load_archive(archive)
    }

    pub fn try_load_archive(archive: SharedZipArchive) -> Result<Self, ModelLoadingError>{
        let model_rdf_bytes: Vec<u8> = 'model_rdf: {
            for file_name in ["rdf.yaml", "bioimageio.yaml"]{
                match archive.read_full_entry(file_name) {
                    Ok(bytes) => break 'model_rdf bytes,
                    Err(zip_err) => match zip_err{
                        zip::result::ZipError::FileNotFound => continue,
                        err => return Err(ModelLoadingError::ZipError(err))
                    }
                };
            }
            return Err(ModelLoadingError::RdfYamlNotFound)
        };
        let model_rdf = match serde_yaml::from_slice::<ModelRdfV0_5>(&model_rdf_bytes){
            Ok(model_rdf) => model_rdf,
            Err(v5_err) => {
                if let Ok(legacy_model) = serde_yaml::from_slice::<UnsupportedLegacyModel>(&model_rdf_bytes){
                    return Err(ModelLoadingError::UnsupportedLegacyModel {
                        version: legacy_model.format_version,
                        earliest_supported: Version_0_5_x::earliest_supported_version(),
                    })
                }
                if let Ok(future_model) = serde_yaml::from_slice::<UnsupportedFutureModel>(&model_rdf_bytes){
                    return Err(ModelLoadingError::FutureModel{
                        format_version: future_model.format_version,
                        latest_supported: Version_0_5_x::latest_supported_version(),
                    })
                }
                return Err(ModelLoadingError::BadModel { inner: v5_err })
            }
        };

        let covers: Vec<CoverImage> = model_rdf.covers.into_iter()
            .map(|rdf_cover| CoverImage::try_load(rdf_cover, &archive))
            .collect::<Result<_, _>>()?;

        let attachments: Vec<FileSource> = model_rdf.attachments.into_iter()
            .map(|att| match att.source{
                rdf::FileReference::Url(_) => return Err(ModelLoadingError::UrlFileReferenceNotSupportedYet),
                rdf::FileReference::Path(fs_path) => {
                    Ok(FileSource::FileInZipArchive { archive: archive.clone(), inner_path: Arc::from(String::from(fs_path).as_str()) })
                }
            })
            .collect::<Result<_, _>>()?;
        let icon = model_rdf.icon.map(|icon| Icon::try_load(icon, &archive)).transpose()?;

        let mut documentation = String::new();
        match model_rdf.documentation{
            rdf::FileReference::Url(_) => return Err(ModelLoadingError::UrlFileReferenceNotSupportedYet),
            FileReference::Path(path) => {
                let path_string: String = path.into();
                archive.with_entry(&path_string, |entry| {
                    entry.read_to_string(&mut documentation)
                })??;
            },
        }
        let weights = ModelWeights::try_from_rdf(model_rdf.weights, archive.clone())?;

        let input_slots: Vec<_> = model_rdf.inputs.into_inner().into_iter()
            .map(|rdf| InputSlot::<Arc<NpyArray>>::try_from_rdf(rdf, archive.clone()))
            .collect::<Result<_, _>>()?;
        let output_slots: Vec<_> = model_rdf.outputs.into_inner().into_iter()
            .map(|rdf| OutputSlot::<Arc<NpyArray>>::try_from_rdf(rdf, archive.clone()))
            .collect::<Result<_, _>>()?;

        let model_interface = ModelInterface::try_build(input_slots, output_slots)?;

        Ok(Self{
            description: model_rdf.description,
            covers,
            attachments,
            cite: model_rdf.cite,
            config: model_rdf.config,
            git_repo: model_rdf.git_repo,
            icon,
            links: model_rdf.links,
            maintainers: model_rdf.maintainers,
            tags: model_rdf.tags,
            version: model_rdf.version,
            version_comment: model_rdf.version_comment,
            authors: model_rdf.authors,
            documentation,
            license: model_rdf.license,
            name: model_rdf.name,
            id: model_rdf.id,
            weights,
            interface: model_interface,
        })
    }
}

impl ZooModel {
    #[cfg(not(target_arch="wasm32"))]
    pub fn pack_into_tmp(self) -> Result<std::fs::File, ModelPackingError>{
        let mut tmp_file = tempfile::tempfile()?;
        self.pack_into(&mut tmp_file)?;
        tmp_file.rewind()?;
        Ok(tmp_file)
    }
    pub fn pack_into<Sink: Write + Seek>(self, sink: Sink) -> Result<(), ModelPackingError> {
        let mut writer = ModelZipWriter::new(sink);

        let (inputs, outputs) = self.interface.dump(&mut writer)?;
        let covers = self.covers.iter().map(|cov| {
            cov.dump(&mut writer)
        }).collect::<Result<Vec<_>, _>>()?;
        let attachments = self.attachments.iter().map(|file|{
            file.dump_as_file_description(&mut writer)
        }).collect::<Result<Vec<_>, _>>()?;
        let icon: Option<rdf::Icon> = match &self.icon{
            Some(icon) => Some(icon.dump(&mut writer)?),
            None => None,
        };
        let documentation: FileReference = {
            let documentation_path = FsPath::unique_suffixed("_README.md");
            writer.write_file(&documentation_path, |writer| -> Result<FileReference, std::io::Error> {
                writer.write_all(self.documentation.as_bytes())?;
                Ok(FileReference::Path(documentation_path.clone()))
            })?
        };
        let timestamp = iso8601_timestamp::Timestamp::now_utc();
        let weights = self.weights.rdf_dump(&mut writer)?;

        let model_rdf = ModelRdfV0_5 {
            description: self.description,
            covers,
            id: None,
            attachments,
            cite: self.cite,
            config: self.config,
            git_repo: self.git_repo,
            icon,
            links: self.links,
            maintainers: self.maintainers,
            tags: self.tags,
            version: self.version,
            version_comment: self.version_comment,
            format_version: Version_0_5_x::new(),
            rdf_type: RdfTypeModel,
            authors: self.authors,
            documentation,
            inputs,
            license: self.license,
            name: self.name,
            outputs,
            run_mode: None,
            timestamp,
            training_data: None, //FIXME
            weights,
        };
        let model_json_val = serde_json::to_value(&model_rdf).unwrap();

        let rdf_file_name = FsPathComponent::try_from("rdf.yaml".to_owned()).unwrap();
        let rdf_path = FsPath::from_components(vec![rdf_file_name]).unwrap();
        writer.write_file(&rdf_path, |writer| serde_yaml::to_writer(writer, &model_json_val))?;

        writer.finish()?;
        Ok(())
    }
}
