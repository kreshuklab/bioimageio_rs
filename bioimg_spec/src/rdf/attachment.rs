use serde::{Serialize, Deserialize};

use super::file_reference::FileReference;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Attachments{
    files: Option<Vec<FileReference>>,
}
