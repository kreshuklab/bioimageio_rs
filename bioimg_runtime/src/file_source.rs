use std::fmt::Display;
use std::sync::Arc;
use std::io::{Read, Seek, Write};
use std::borrow::Borrow;

use bioimg_spec::rdf::{self, FileReference, HttpUrl};

use crate::{zip_archive_ext::SharedZipArchive, zip_writer_ext::ModelZipWriter, zoo_model::ModelPackingError};

#[derive(thiserror::Error, Debug)]
pub enum FileSourceError{
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("IO error trying to read {path}: {inner}")]
    ZipError{inner: zip::result::ZipError, path: String},
    #[error("Error downloading file: {reason}")]
    HttpError{reason: String}
}

#[derive(Clone, Debug)]
pub enum FileSource{
    Data{data: Arc<[u8]>, name: Option<String>},
    #[cfg(not(target_arch="wasm32"))]
    LocalFile{path: Arc<std::path::Path>},
    FileInZipArchive{archive: SharedZipArchive, inner_path: Arc<str>},
    HttpUrl(Arc<HttpUrl>),
}

impl PartialEq for FileSource{
    fn eq(&self, other: &Self) -> bool {
        match (self, other){
            #[cfg(not(target_arch="wasm32"))]
            (
                Self::LocalFile{path: p_self},
                Self::LocalFile { path: p_other }
            ) => p_self == p_other,
            (
                Self::FileInZipArchive{archive: arch_self, inner_path: path_self},
                Self::FileInZipArchive{archive: arch_other, inner_path: path_other }
            ) => {
                arch_self == arch_other && path_self == path_other
            },
            (Self::HttpUrl(self_url), Self::HttpUrl(other_url)) => self_url == other_url,
            _ => false
        }
    }
}
impl Eq for FileSource{}

impl Display for FileSource{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Data{name, data} => {
                if let Some(name) = name {
                    write!(f, "{name} ")?;
                }
                write!(f, "{} bytes", data.len()) //FIXME: KB, MB, GB ?
            },
            #[cfg(not(target_arch="wasm32"))]
            Self::LocalFile { path } => write!(f, "{}", path.to_string_lossy()),
            Self::FileInZipArchive { inner_path, .. } => write!(f, "*.zip/{inner_path}"), //FIXME? *.zip?
            Self::HttpUrl(http_url) => write!(f, "{}", http_url.as_str()),
        }
    }
}

impl FileSource{
    //FIXME: add some cancellation token?
    fn rdf_dump(
        &self,
        zip_file: &mut ModelZipWriter<impl Write + Seek>,
    ) -> Result<rdf::FsPath, ModelPackingError> {
        let extension = match self{
            Self::Data{name, ..} => if let Some(name) = name {
                name.split(".").last().map(|s| s.to_owned())
            } else {
                None
            }
            #[cfg(not(target_arch="wasm32"))]
            Self::LocalFile { path } => path.extension().map(|ex| ex.to_string_lossy().to_string()),
            Self::FileInZipArchive { inner_path, .. } => {
                inner_path.split(".").last().map(|s| s.to_owned())
            },
            Self::HttpUrl(url) => {
                url.path().split(".").last().map(|s| s.to_owned())
            }
        };
        let output_inner_path = match extension{
            Some(ext) => rdf::FsPath::unique_suffixed(&format!(".{ext}")),
            None => rdf::FsPath::unique(),
        };
        zip_file.write_file(&output_inner_path, |writer| -> Result<u64, ModelPackingError>{
            let copied_bytes: u64 = match self{
                Self::Data{ data, .. } => {
                    let mut reader = std::io::Cursor::new(&data);
                    std::io::copy(&mut reader, writer)?
                },
                #[cfg(not(target_arch="wasm32"))]
                Self::LocalFile { path } => {
                    std::io::copy(&mut std::fs::File::open(path)?, writer)?
                },
                Self::FileInZipArchive { archive, inner_path } => {
                    archive.with_entry(&inner_path, |entry|{
                        std::io::copy(entry, writer)
                    })??
                },
                #[cfg(target_arch = "wasm32")]
                Self::HttpUrl(_http_url) => {
                    return Err(ModelPackingError::HttpErro { reason: "Downloading in wasm not implemented yet".to_owned() })
                }
                #[cfg(not(target_arch = "wasm32"))]
                Self::HttpUrl(http_url) => {
                    let response = ureq::get(http_url.as_str()).call()
                        .map_err(|e| ModelPackingError::HttpErro { reason: e.to_string()})?;
                    eprintln!("Requesting {http_url} returned result {}", response.status());
                    if response.status() / 100 != 2{
                        return Err(ModelPackingError::UnexpectedHttpStatus {
                            status: response.status(),
                            url: http_url.as_ref().clone(),
                        })
                    }
                    let mut response_reader = response.into_reader();
                    std::io::copy(&mut response_reader, writer)? //FIXME!! limit size or whatever
                }
            };
            Ok(copied_bytes)
        })?;
        Ok(output_inner_path)
    }

    pub fn rdf_dump_as_file_reference(
        &self,
        zip_file: &mut ModelZipWriter<impl Write + Seek>,
    ) -> Result<rdf::FileReference, ModelPackingError> {
        let output_inner_path = self.rdf_dump(zip_file)?;
        Ok(rdf::FileReference::Path(output_inner_path))
    }

    pub fn dump_as_file_description(
        &self,
        zip_file: &mut ModelZipWriter<impl Write + Seek>,
    ) -> Result<rdf::FileDescription, ModelPackingError> {
        let file_reference = self.rdf_dump_as_file_reference(zip_file)?;
        Ok(rdf::FileDescription{source: file_reference, sha256: None})
    }
}

impl FileSource{
    pub fn from_rdf_file_descr<T: Borrow<FileReference>>(
        archive: SharedZipArchive, file_reference: &rdf::FileDescription<T>
    ) -> Result<Self, FileSourceError>{
        Self::from_rdf_file_reference(archive, file_reference.source.borrow())
    }


    pub fn from_rdf_file_reference(
        archive: SharedZipArchive, file_reference: &rdf::FileReference
    ) -> Result<Self, FileSourceError>{
        Ok(match file_reference{
            rdf::FileReference::Url(url) => Self::HttpUrl(Arc::new(url.clone())),
            rdf::FileReference::Path(path) => {
                let path = String::from(path);
                archive.with_entry(&path, |_| {}).map_err(|e|{
                    FileSourceError::ZipError{inner: e, path: path.clone()}
                })?;
                Self::FileInZipArchive {
                    archive,
                    inner_path:  Arc::from(path.as_str())
                }
            }
        })
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> Result<usize, FileSourceError>{
        match self{
            Self::Data { data, .. } => {
                let mut reader = std::io::Cursor::new(data);
                Ok(reader.read_to_end(buf)?)
            }
            #[cfg(not(target_arch="wasm32"))]
            Self::LocalFile { path } => Ok(std::fs::File::open(path)?.read_to_end(buf)?),
            Self::FileInZipArchive { archive, inner_path } => {
                let bytes_read = archive.with_entry(&inner_path, |entry| entry.read_to_end(buf))
                    .map_err(|inner| FileSourceError::ZipError { inner, path: inner_path.as_ref().to_owned()})??;
                Ok(bytes_read)
            },
            #[cfg(target_arch = "wasm32")]
            Self::HttpUrl(_http_url) => {
                Err(FileSourceError::HttpError { reason: "Can't download on was yet; needs to be async".to_owned() })
            },
            #[cfg(not(target_arch = "wasm32"))]
            Self::HttpUrl(http_url) => {
                let mut response_reader = ureq::get(http_url.as_str())
                .call()
                .map_err(|e| FileSourceError::HttpError { reason: e.to_string()})?
                .into_reader();
                Ok(response_reader.read_to_end(buf)?)
            }
        }
    }
}
