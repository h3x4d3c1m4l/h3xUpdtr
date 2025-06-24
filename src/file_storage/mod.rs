use std::{collections::HashMap, io::Read, path::Path};

use snafu::Snafu;

pub mod s3;

pub trait FileStore {
    async fn upload_file<T: Read>(&self, relative_path: &Path, data_stream: T, metadata: HashMap<&str, &str>) -> FileStoreResult<(), FileStoreError>;
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
