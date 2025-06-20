
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDefinition {
    /// Relative path of the file.
    pub r_path: String,

    /// File size of the uncompressed file.
    pub u_size: u64,

    /// SHA256 hash of the uncompresed file.
    pub u_sha256: String,

    /// SHA256 hash of the compressed file.
    pub c_sha256: String,
}
