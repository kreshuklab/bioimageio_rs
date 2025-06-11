pub mod attachment;
pub mod author;
pub mod badge;
pub mod bounded_string;
pub mod cite_entry;
pub mod clamped;
pub mod file_reference;
pub mod file_description;
pub mod icon;
pub mod identifier;
pub mod license;
pub mod literal;
pub mod lowercase;
pub mod maintainer;
pub mod model;
pub mod non_empty_list;
pub mod orcid;
pub mod si_units;
pub mod slashless_string;
pub mod basic_chars_string;
pub mod version;
pub mod tag;

pub use bounded_string::BoundedString;
pub use icon::{EmojiIcon, Icon, IconParsingError};
pub use identifier::Identifier;
pub use license::LicenseId;
pub use literal::{LiteralInt, LitStr};
pub use version::Version;
pub use file_reference::{HttpUrl, FsPath, FileReference, CoverImageSource, EnvironmentFile};
pub use author::Author2;
pub use file_description::{FileDescription, EnvironmentFileDescr};
pub use maintainer::{Maintainer, MaintainerName};
pub use orcid::Orcid;
pub use cite_entry::CiteEntry2;
pub use tag::Tag;
pub use non_empty_list::NonEmptyList;

use crate::util::AsPartial;

use self::{lowercase::Lowercase, slashless_string::SlashlessString};

pub type ResourceId = SlashlessString<Lowercase<BoundedString<1, 1024>>>;
pub type ResourceTextDescription = BoundedString<0, 1024>;

impl AsPartial for ResourceTextDescription{
    type Partial = String;
}
