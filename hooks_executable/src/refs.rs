use std::str::FromStr;

use camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug)]
pub enum GitRef{
    Head(GitHeadRef),
    Tag(GitTagRef),
}

#[derive(Debug)]
pub struct GitTagRef{
    path: Utf8PathBuf,
}
impl GitTagRef{
    pub fn name(&self) -> &str {
        self.path.file_name().unwrap()
    }
    pub fn version(&self) -> Option<versions::Version>{
        let name = self.name();
        let name = name.strip_prefix("v")?;
        if !name.chars().next()?.is_numeric(){
            return None
        }
        name.parse::<versions::Version>().ok()
    }
}

#[derive(Debug)]
pub struct GitHeadRef{
    path: Utf8PathBuf,
}

impl GitRef{
    pub fn path(&self) -> &Utf8Path {
        match self{
            Self::Head(head) => &head.path,
            Self::Tag(tag) => &tag.path,
        }
    }
    pub fn name(&self) -> &str {
        self.path().file_name().unwrap()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GitRefParsingError{
    #[error("Value has no path components")]
    NoComponents,
    #[error("Value doesn't start with a 'refs' component: {0}")]
    DoesntStartWithRefs(String),
    #[error("Value doesn't have a component after 'refs' indicating a category")]
    NoCategory,
    #[error("Git ref cateogry not regognized: {0}")]
    CategoryParsingError(String),
    #[error("Refenrece path has no name (too few components)")]
    NoRefName,
}

impl FromStr for GitRef{
    type Err = GitRefParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = Utf8Path::new(s);
        let mut components = path.components();
        let first_component = components.next().ok_or(Self::Err::NoComponents)?;
        if first_component.as_str() != "refs" {
            return Err(Self::Err::DoesntStartWithRefs(path.as_str().to_owned()))
        }
        let raw_ref_category = components.next().ok_or(Self::Err::NoCategory)?;
        if components.next().is_none(){
            return Err(Self::Err::NoCategory);
        }
        Ok(match raw_ref_category.as_str() {
            "tags" => Self::Tag(GitTagRef{ path: path.to_owned() }),
            "heads" => Self::Head(GitHeadRef{ path: path.to_owned() }),
            _ => {
                return Err(Self::Err::CategoryParsingError(raw_ref_category.as_str().to_owned()))
            },
        })
    }
}


