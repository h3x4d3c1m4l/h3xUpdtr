use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDefinition {
    pub version: DefinitionVersion,
    pub files: Vec<FileDefinition>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u32)]
pub enum DefinitionVersion {
   Version1 = 1,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDefinition {
    /// Relative path of the file.
    pub r_path: String,

    /// File size of the uncompressed file.
    pub u_len: u32,

    /// SHA256 hash of the uncompresed file.
    pub u_sha256: String,

    /// Algorithm used for compressing the data.
    pub c_algo: String,

    /// File size of the compressed file.
    pub c_len: u32,

    /// SHA256 hash of the compressed file.
    pub c_sha256: String,
}
