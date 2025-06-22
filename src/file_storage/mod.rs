use std::io::Read;

use snafu::Snafu;

pub mod s3;

pub trait FileStore {
    async fn create_file<T: Read>(self, relative_path: String, data_stream: T) -> FileStoreResult<(), FileStoreError>;
}

// ////// //
// Errors //
// ////// //

#[derive(Debug, Snafu)]
enum FileStoreError {
    #[snafu(display("Unable to store the file"))]
    CreateFileError,
}

pub type FileStoreResult<T, E = FileStoreError> = std::result::Result<T, E>;
