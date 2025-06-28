use std::{collections::HashMap, io::Read, path::Path};

use snafu::Snafu;
use futures::stream::BoxStream;
use bytes::Bytes;

pub mod s3;

pub trait FileStore {
    async fn upload_file<T: Read>(&self, relative_path: &Path, data_stream: T, metadata: HashMap<&str, &str>) -> FileStoreResult<(), FileStoreError>;
    async fn get_file_info(&self, relative_path: &Path) -> FileStoreResult<Option<RemoteFileInfo>, FileStoreError>;
    async fn get_file(&self, relative_path: &Path) -> FileStoreResult<Option<RemoteFile>, FileStoreError>;
}

#[derive(Debug)]
pub struct RemoteFileInfo {
    pub c_len: u32,
    pub metadata: HashMap<String, String>,
}

pub struct RemoteFile {
    pub c_len: u32,
    pub metadata: HashMap<String, String>,
    pub stream: BoxStream<'static, Bytes>,
}

// ////// //
// Errors //
// ////// //

#[derive(Debug, Snafu)]
pub enum FileStoreError {
    #[snafu(display("Unable to store the file"))]
    CreateFileError,
}

pub type FileStoreResult<T, E = FileStoreError> = std::result::Result<T, E>;
