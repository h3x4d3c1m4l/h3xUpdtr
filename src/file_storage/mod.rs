use std::io::Read;

use snafu::Snafu;

pub mod s3;

pub trait FileStore {
    async fn upload_file<T: Read>(&self, relative_path: &str, data_stream: T) -> FileStoreResult<(), FileStoreError>;
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
