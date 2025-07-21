use std::{fmt::Display, ops::Deref, path::PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

use aspartial::AsPartial;


#[derive(thiserror::Error, Debug)]
pub enum FsPathParsingError{
    #[error("Bad character in file path component '{raw}'")]
    BadComponentChar{raw: String},
    #[error("Empty path components")]
    EmptyComponent{raw: String},
    #[error("Empty path")]
    EmptyPath,
    #[error("Path is not relative: {0}")]
    PathNotRelative(String)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct FsPathComponent(String);

impl Deref for FsPathComponent{
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FsPathComponent{
    pub fn unique() -> Self{
        Self( uuid::Uuid::new_v4().to_string() )
    }
    pub fn unique_suffixed(suffix: &str) -> Self{
        Self( format!("{}{suffix}", uuid::Uuid::new_v4().to_string()) )
    }
}

impl TryFrom<String> for FsPathComponent{
    type Error = FsPathParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains(['/', '\\']){
            return Err(FsPathParsingError::BadComponentChar { raw: value })
        }
        if value.len() == 0{
            return Err(FsPathParsingError::EmptyComponent { raw: value })
        }
        Ok(Self(value))
    }
}

impl From<FsPathComponent> for String{
    fn from(value: FsPathComponent) -> Self {
        value.0
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct FsPath{
    components: Vec<FsPathComponent>
}

impl AsPartial for FsPath {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.into()
    }
}

impl FsPath{
    pub fn from_components(components: Vec<FsPathComponent>) -> Result<Self, FsPathParsingError>{
        if components.len() == 0{
            Err(FsPathParsingError::EmptyPath)
        }else{
            Ok(Self{components})
        }
    }
    pub fn unique() -> Self{
        Self{ components: vec![ FsPathComponent::unique() ] }
    }
    pub fn unique_suffixed(suffix: &str) -> Self{
        Self{ components: vec![ FsPathComponent::unique_suffixed(suffix) ] }
    }
    pub fn components(&self) -> &[FsPathComponent]{
        &self.components
    }
    pub fn file_name(&self) -> &FsPathComponent{
        self.components.last().as_ref().unwrap()
    }
}

impl TryFrom<String> for FsPath{
    type Error = FsPathParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.starts_with("/"){
            return Err(FsPathParsingError::PathNotRelative(value))
        }
        let components = value.split("/")
            .map(|comp| FsPathComponent::try_from(comp.to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        if components.len() == 0{
            return Err(FsPathParsingError::EmptyPath)
        }
        Ok(Self{components})
    }
}

impl From<FsPath> for String{
    fn from(value: FsPath) -> Self {
        (&value).into()
    }
}

impl From<&FsPath> for String{
    fn from(value: &FsPath) -> Self {
        let mut out = String::with_capacity(
            value.components.iter().map(|comp| comp.0.len()).sum::<usize>() + value.components.len()
        );
        for (comp_idx, comp) in value.components.iter().enumerate(){
            if comp_idx != 0{
                out += "/"
            }
            out += &comp.0;
        }
        out
    }
}

impl From<&FsPath> for PathBuf{
    fn from(value: &FsPath) -> PathBuf {
        let path_string: String = value.into();
        PathBuf::from(path_string)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum UrlParsingError{
    #[error("{0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Url is not http: {url}")]
    NotHttp{url: url::Url},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct HttpUrl(url::Url);

impl AsPartial for HttpUrl{
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.into()
    }
}

impl HttpUrl{
    pub fn path(&self) -> &str{
        self.0.path()
    }
}

impl Display for HttpUrl{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for HttpUrl{
    type Target = url::Url;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<String> for HttpUrl{
    type Error = UrlParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let url = url::Url::parse(&value)?;
        return Self::try_from(url)
    }
}

impl TryFrom<Url> for HttpUrl {
    type Error = UrlParsingError;
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        if url.scheme() != "http" && url.scheme() != "https"{
            return Err(UrlParsingError::NotHttp { url })
        }
        Ok(Self(url))
    }
}

impl From<HttpUrl> for String{
    fn from(value: HttpUrl) -> Self {
        value.0.into()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FileReferenceParsingError{
    #[error("Could not parse {0} as either a URL or an absolute path")]
    NorUrlNorPath(String),
    #[error("{0}")]
    UrlParsingError(#[from] UrlParsingError),
    #[error("'{raw}' should have one of the following suffixes: {suffixes:?}")]
    BadSuffix{raw: FileReference, suffixes: Vec<&'static str>}
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(untagged)]
pub enum FileReference {
    Url(HttpUrl),
    Path(FsPath),
}

impl AsPartial for FileReference {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        match self{
            Self::Url(url) => url.into(),
            Self::Path(path) => path.into(),
        }
    }
}

impl std::fmt::Display for FileReference{
    //FIXME: maybe the default impl of Into<String> should be here to avoid clones
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let raw = match self{
            Self::Path(path) => Into::<String>::into(path.clone()),
            Self::Url(url) => Into::<String>::into(url.clone()),
        };
        write!(f, "{}", raw)
    }
}

impl From<HttpUrl> for FileReference{
    fn from(value: HttpUrl) -> Self {
        Self::Url(value)
    }
}
impl From<FsPath> for FileReference{
    fn from(value: FsPath) -> Self {
        Self::Path(value)
    }
}

impl TryFrom<String> for FileReference{
    type Error = FileReferenceParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Ok(parsed) = Url::parse(&value){
            let http_url = HttpUrl::try_from(parsed)?;
            return Ok(Self::Url(http_url))
        }
        if let Ok(parsed) = FsPath::try_from(value.clone()){
            return Ok(Self::Path(parsed))
        }
        return Err(FileReferenceParsingError::NorUrlNorPath(value))
    }
}


macro_rules! suffixed_file_ref {(
    struct $name:ident suffixes=[ $($suffix:literal),+ ]
) => {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Clone)]
    pub struct $name(FileReference);

    impl TryFrom<FileReference> for $name{
        type Error = FileReferenceParsingError;
        fn try_from(value: FileReference) -> Result<Self, Self::Error> {
            let raw: String = match &value{
                FileReference::Path(path) => path.clone().into(),
                FileReference::Url(url) => url.path().into(),
            };
            let raw = raw.to_lowercase();
            for suf in [ $($suffix),+ ]{
                if raw.ends_with(suf){
                    return Ok(Self(value))
                }
            }
            Err(FileReferenceParsingError::BadSuffix{
                raw: value,
                suffixes: Vec::from( [ $($suffix),+ ] )
            })
        }
    }

    impl Deref for $name{
        type Target = FileReference;
        fn deref(&self) -> &Self::Target{
            &self.0
        }
    }

    impl core::borrow::Borrow<FileReference> for $name{
        fn borrow(&self) -> &FileReference{
            &self.0
        }
    }

    impl ::aspartial::AsPartial for $name {
        type Partial = String;
        fn to_partial(self) -> Self::Partial {
            self.0.to_partial()
        }
    }
};}

suffixed_file_ref!(struct CoverImageSource suffixes=[".gif", ".jpeg", ".jpg", ".png"]);
suffixed_file_ref!(struct EnvironmentFile suffixes=[".yaml", ".yml"]);

#[test]
fn test_file_reference() {
    use serde_json::json;

    let raw_url = "http://bla/ble?lalala";
    let deserialized_url: FileReference = serde_json::from_value(json!(raw_url)).unwrap();
    assert_eq!(FileReference::Url(HttpUrl::try_from(raw_url.to_owned()).unwrap()), deserialized_url,);

    let raw_path = "lalala/lelele";
    dbg!(FsPath::try_from(raw_path.to_owned()).unwrap());
    let deserialized_path: FileReference = serde_json::from_value(json!(raw_path)).unwrap();
    assert_eq!(FileReference::Path(FsPath::try_from(raw_path.to_owned()).unwrap()), deserialized_path,);
}
