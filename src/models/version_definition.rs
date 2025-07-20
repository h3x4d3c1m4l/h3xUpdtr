use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// The definition of a version.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDefinition {
    /// The version of the definition schema. Always `1` for now.
    pub version: DefinitionVersion,

    /// The display version of the application. This is only used for cosmetic purpose and can be `null`.
    ///
    /// e.g. "1.2.3".
    pub display_version: Option<String>,

    /// The files this version consists of.
    pub files: Vec<FileDefinition>,
}

/// The definition schema versions.
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u32)]
pub enum DefinitionVersion {
    /// Version 1.
    Version1 = 1,
}

/// The definition of a version's file.
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
