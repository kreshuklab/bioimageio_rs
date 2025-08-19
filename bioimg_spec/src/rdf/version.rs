use std::str::FromStr;

use aspartial::AsPartial;


#[derive(thiserror::Error, Debug)]
pub enum VersionParsingError {
    #[error(transparent)]
    BadVersionString{#[from] source: versions::Error},
    #[error("Version '{version}' is too low")]
    TooLow{ version: Version },
    #[error("Version '{version}' is too high. Max supported is {max_supported}")]
    TooHigh{ version: Version, max_supported: Version },
}

#[derive(
    PartialOrd, Ord, Clone, Debug, PartialEq, Eq,
    serde::Deserialize, serde::Serialize,
    derive_more::Display, derive_more::Deref, derive_more::FromStr,
)]
#[serde(try_from="String")]
#[serde(into="String")]
pub struct Version(versions::Version);

impl AsPartial for Version {
    type Partial = String;
    fn to_partial(self) -> Self::Partial {
        self.0.to_string()
    }
}

impl Version{
    pub fn major_minor_patch(major: u32, minor: u32, patch: u32) -> Self{
        Version(versions::Version{
            chunks: versions::Chunks(vec![
                versions::Chunk::Numeric(major),
                versions::Chunk::Numeric(minor),
                versions::Chunk::Numeric(patch),
            ]),
            ..Default::default()
        })
    }
    pub fn version_0_5_3() -> Version{
        Self::major_minor_patch(0, 5, 3)
    }
    pub fn version_0_5_0() -> Version{
        Self::major_minor_patch(0, 5, 0)
    }
}

impl TryFrom<String> for Version{
    type Error = VersionParsingError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let inner = versions::Version::from_str(&value)?;
        Ok(Self(inner))
    }
}

impl From<Version> for String{
    fn from(value: Version) -> Self {
        value.0.to_string()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, AsPartial)]
#[aspartial(newtype)]
#[serde(try_from="Version")]
pub struct Version_0_5_x(Version);

impl Version_0_5_x{
    pub fn new() -> Self{
        Self(Version::version_0_5_3())
    }
    pub fn latest_supported_version() -> Version{
        Version::version_0_5_3()
    }
    pub fn earliest_supported_version() -> Version{
        Version::version_0_5_0()
    }
}

impl TryFrom<Version> for Version_0_5_x {
    type Error = VersionParsingError;
    fn try_from(version: Version) -> Result<Self, Self::Error> {
        if  version < Version::version_0_5_0() {
            return Err(VersionParsingError::TooLow { version })
        }
        if  version > Version::version_0_5_3() {
            return Err(VersionParsingError::TooHigh { version, max_supported: Version::version_0_5_3() })
        }
        Ok(Self(version))
    }
}
