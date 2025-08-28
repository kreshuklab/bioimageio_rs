use std::{fmt::{Debug, Display}, io::{Read, Seek}, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use bioimg_spec::rdf;
// use zip::{read::ZipFile, ZipArchive};


pub trait SeekReadSend: Seek + Read + Send{}
impl<T: Seek + Read + Send> SeekReadSend for T{}

type BoxDynSeekReadSend = Box<dyn SeekReadSend + 'static>;
type AnyZipArchive = zip::ZipArchive<BoxDynSeekReadSend>;

/// Something that uniquely identifies a zip archive
///
/// Either its path if it lives in the fs, or a name if its, say, in memory
#[derive(Clone, Debug)]
pub enum ZipArchiveIdentifier{
    Path(PathBuf),
    /// For archives that don't live in the file system, like on memory or other web abstraction
    Name(String),
}

impl From<PathBuf> for ZipArchiveIdentifier{
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl From<String> for ZipArchiveIdentifier{
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

impl Display for ZipArchiveIdentifier{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::Path(p) => write!(f, "{}", p.to_string_lossy()),
            Self::Name(name) => write!(f, "{name}")
        }
    }
}

#[derive(Clone)]
pub struct SharedZipArchive{
    identif: ZipArchiveIdentifier,
    archive: Arc<Mutex<AnyZipArchive>>,
}

impl Debug for SharedZipArchive{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SharedZipArchive{{ identif: {:?} }}", self.identif)
    }
}

impl PartialEq for SharedZipArchive{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.archive, &other.archive)
    }
}
impl Eq for SharedZipArchive{}

pub enum ZipArchiveError<E>{
    ZipError(zip::result::ZipError),
    Other(E)
}

#[derive(thiserror::Error, Debug)]
pub enum ZipArchiveOpenError{
    #[error("Could read zip archive file: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError)
}

impl SharedZipArchive{
    pub fn identifier(&self) -> &ZipArchiveIdentifier{
        &self.identif
    }
    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self, ZipArchiveOpenError>{
        let file: Box<dyn SeekReadSend + 'static> = Box::new(std::fs::File::open(p.as_ref())?);
        let archive: AnyZipArchive = zip::ZipArchive::new(file)?;
        Ok(Self{
            identif: ZipArchiveIdentifier::Path(p.as_ref().to_owned()),
            archive: Arc::new(Mutex::new(archive))
        })
    }
    pub fn new(identif: ZipArchiveIdentifier, archive: AnyZipArchive) -> Self{
        Self{identif, archive: Arc::new(Mutex::new(archive))}
    }
    pub fn from_raw_data(contents: Vec<u8>, ident: impl Into<ZipArchiveIdentifier>) -> Self{
        let reader: Box<dyn SeekReadSend + 'static> = Box::new(std::io::Cursor::new(contents));
        let archive = zip::ZipArchive::new(reader).unwrap();
        SharedZipArchive::new(
            ident.into(),
            archive
        )
    }
    pub fn with_entry<F, Out>(&self, name: &str, entry_reader: F) -> Result<Out, zip::result::ZipError>
    where
        F: FnOnce(&mut zip::read::ZipFile<'_, BoxDynSeekReadSend>) -> Out,
        Out: 'static,
    {
        let mut archive_guard = self.archive.lock().unwrap();
        let mut f = archive_guard.by_name(name)?;
        let out = entry_reader(&mut f);
        Ok(out)
    }
    pub fn read_full_entry(&self, entry_path: &str) -> Result<Vec<u8>, zip::result::ZipError>{
        let mut archive_guard = self.archive.lock().unwrap();
        let mut f = archive_guard.by_name(entry_path)?;
        let mut data = Vec::<u8>::new();
        _ = f.read_to_end(&mut data)?;
        Ok(data)
    }
    pub fn has_entry(&self, name: &str) -> bool{
        self.archive.lock().unwrap().by_name(name).is_ok()
    }
    pub fn with_file_names<F, Out>(&self, f: F) -> Out
    where
        F: for<'a> FnOnce(Box<dyn Iterator<Item=&'a str> + 'a>) -> Out,
        Out: 'static,
    {
        let archive_guard = self.archive.lock().unwrap();
        let file_names = Box::new(archive_guard.file_names());
        f(file_names)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RdfFileReferenceReadError{
    #[error("{0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("Url file reference not supported yet")]
    UrlFileReferenceNotSupportedYet,
}

pub trait RdfFileReferenceExt{
    fn try_read<F, Out>(
        &self, archive: &SharedZipArchive, reader: F
    ) -> Result<Out, RdfFileReferenceReadError>
    where
        F: FnOnce(&mut zip::read::ZipFile<'_, BoxDynSeekReadSend>) -> Out,
        Out: 'static;
}
impl RdfFileReferenceExt for rdf::FileReference{
    fn try_read<F, Out>(&self, archive: &SharedZipArchive, reader: F) -> Result<Out, RdfFileReferenceReadError>
    where
        F: FnOnce(&mut zip::read::ZipFile<'_, BoxDynSeekReadSend>) -> Out,
        Out: 'static,
    {
        let inner_path: String = match self{
            rdf::FileReference::Url(_) => return Err(RdfFileReferenceReadError::UrlFileReferenceNotSupportedYet),
            rdf::FileReference::Path(path) => path.into(),
        };
        Ok(archive.with_entry(&inner_path, reader)?)
    }
}
